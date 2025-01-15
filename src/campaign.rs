use derive_more::derive::{Deref, DerefMut};
use rand::RngCore;
use std::collections::BTreeSet;

use bevy::prelude::{
    any_with_component, Added, App, Changed, Commands, Component, Entity, Event, IntoSystemConfigs,
    Name, Plugin, Query, Single, SystemSet, Trigger, Update, With, Without,
};
use bevy_prng::WyRand;
use bevy_rand::{global::GlobalEntropy as BRGlobalEntropy, plugin::EntropyPlugin};

use crate::{
    ui::{AppScreen, BonusUIComponent, GameOverUIComponent, GameUIComponent},
    Aspect, Augmentation, Element, ElementTable, Game, GameOver, Guess, Health, InGame, Payout,
    Player, PlayerElements, SpawnGame,
};

type GlobalEntropy<'w> = BRGlobalEntropy<'w, WyRand>;

pub struct CampaignPlugin;

impl Plugin for CampaignPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EntropyPlugin::<WyRand>::default());
        app.add_observer(SpawnCampaign::observer);
        app.add_systems(
            Update,
            (
                Self::change_phases,
                Self::detect_game_over,
                Self::make_enemy_guesses,
            )
                .run_if(any_with_component::<Campaign>)
                .chain()
                .in_set(CampaignSystems),
        );
    }
}

impl CampaignPlugin {
    #[allow(clippy::type_complexity)]
    fn change_phases(
        mut commands: Commands,
        mut rng: GlobalEntropy,
        campaign: Option<
            Single<
                (Entity, &Level, &Phase, &Round, &ElementTable),
                (Changed<Phase>, With<Campaign>),
            >,
        >,
        player: Single<(Entity, &PlayerElements), With<CampaignPlayer>>,
        mut app_screen: Single<&mut AppScreen>,
    ) {
        let Some(campaign) = campaign else {
            return;
        };
        let level = campaign.1;
        let phase = campaign.2;
        let (player, elements) = (player.0, player.1);

        match phase {
            Phase::InGame => {
                let enemy_elements =
                    PlayerElements::from_set(Element::random_set(&mut **rng, level.num_elements()));
                let enemy = commands
                    .entity(campaign.0)
                    .insert((Health::new(5), enemy_elements.clone(), Player))
                    .id();

                commands.trigger(SpawnGame {
                    player_one: Some(player),
                    player_two: Some(enemy),
                    table: Some(campaign.4.clone()),
                    ..Default::default()
                });

                **app_screen = AppScreen::InGame(GameUIComponent {
                    player,
                    player_elements: elements.clone(),
                    enemy_elements,
                    table: campaign.4.clone(),
                    round: *campaign.3,
                });
                //  (player, elements.clone(), *campaign.4));
            }
            Phase::ProvidingBonus => {
                // calculate two bonus options for the upgrade option
                let possible_bonus_elements = elements.elements.iter().cloned().collect::<Vec<_>>();
                let bonus_element_1 =
                    Element::random_item(&mut **rng, possible_bonus_elements.as_slice());
                let bonus_element_2 =
                    Element::random_item(&mut **rng, possible_bonus_elements.as_slice());
                let bonus_element_3 =
                    Element::random_item(&mut **rng, possible_bonus_elements.as_slice());
                let bonus_choices = [
                    (
                        Bonus::new_random(bonus_element_1, &mut **rng),
                        bonus_element_1,
                    ),
                    (
                        Bonus::new_random(bonus_element_2, &mut **rng),
                        bonus_element_2,
                    ),
                    (
                        Bonus::new_random(bonus_element_3, &mut **rng),
                        bonus_element_3,
                    ),
                ];
                // calculate two new elements for the evolution option
                let all_elements = BTreeSet::from(Element::ALL);
                let remaining_elements = all_elements
                    .difference(&elements.elements)
                    .cloned()
                    .collect::<Vec<_>>();
                let evolution_choices =
                    Element::random_subset(&mut **rng, remaining_elements.as_slice(), 2);

                **app_screen = AppScreen::ProvidingBonus(BonusUIComponent::new(
                    campaign.4.clone(),
                    Upgrades {
                        bonuses: bonus_choices.to_vec(),
                        evolutions: evolution_choices,
                    },
                ));
            }
            Phase::GameOver => {
                **app_screen = AppScreen::GameOver(GameOverUIComponent::loss(
                    campaign.0,
                    player,
                    *campaign.3,
                    *campaign.1,
                ));
            }
            Phase::Victory => {
                **app_screen = AppScreen::GameOver(GameOverUIComponent::victory(
                    campaign.0,
                    player,
                    *campaign.3,
                    *campaign.1,
                ));
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn detect_game_over(
        mut commands: Commands,
        mut campaign: Single<(&mut Phase, &mut Round, &Campaign, &Level)>,
        game_overs: Option<Single<(Entity, &GameOver), (With<Game>, Added<GameOver>)>>,
    ) {
        let Campaign { player } = campaign.2;
        let player = *player;

        let Some((game_entity, game_over)) = game_overs.as_deref() else {
            return;
        };

        match game_over {
            GameOver::Winner(entity) => {
                if *entity == player {
                    if matches!(campaign.3, Level::Seven) {
                        *campaign.0 = Phase::Victory;
                    } else {
                        *campaign.0 = Phase::ProvidingBonus;
                        **campaign.1 += 1;
                    }
                } else {
                    *campaign.0 = Phase::GameOver;
                }
            }
            GameOver::Draw => {
                *campaign.0 = Phase::GameOver;
            }
        }

        commands.entity(*game_entity).despawn();
        commands.entity(player).remove::<InGame>();
    }

    #[allow(clippy::type_complexity)]
    fn make_enemy_guesses(
        mut commands: Commands,
        mut rng: GlobalEntropy,
        ai_players: Query<(Entity, &PlayerElements), (Without<CampaignPlayer>, Without<Guess>)>,
    ) {
        for (player, elements) in &ai_players {
            let elements = elements.elements.iter().cloned().collect::<Vec<_>>();
            commands
                .entity(player)
                .insert(Guess::new(Element::random_item(
                    &mut **rng,
                    elements.as_slice(),
                )));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct CampaignSystems;

#[derive(Debug)]
#[derive(Default)]
#[derive(Event)]
pub struct SpawnCampaign {
    pub player: Option<Entity>,
}

impl SpawnCampaign {
    fn observer(trigger: Trigger<Self>, mut commands: Commands) {
        let player = trigger
            .event()
            .player
            .unwrap_or_else(|| commands.spawn_empty().id());
        let campaign = commands.spawn(Campaign { player }).id();
        commands
            .entity(player)
            .insert((Health::PLAYER_MAX, CampaignPlayer, InCampaign(campaign)))
            .insert_if_new(Player);
    }
}

#[derive(Component)]
struct CampaignPlayer;

#[derive(Deref, DerefMut)]
#[derive(Component)]
struct InCampaign(pub Entity);

#[derive(Debug)]
#[derive(Component)]
#[require(Level, Round, Phase, ElementTable, PlayerElements, Name(|| Name::new("Campaign")))]
pub struct Campaign {
    pub player: Entity,
}

#[derive(Clone, Copy, Debug)]
#[derive(Deref, DerefMut)]
#[derive(Component)]
pub struct Round(u16);

impl Default for Round {
    fn default() -> Self {
        Round(1)
    }
}

#[derive(Clone, Copy, Default, Debug)]
#[derive(Component)]
pub enum Level {
    #[default]
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl Level {
    pub fn increment(&mut self) {
        use Level::*;
        *self = match self {
            Three => Four,
            Four => Five,
            Five => Six,
            Six => Seven,
            Seven => Three,
        }
    }

    pub fn num_elements(&self) -> usize {
        match self {
            Level::Three => 3,
            Level::Four => 4,
            Level::Five => 5,
            Level::Six => 6,
            Level::Seven => 7,
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
#[derive(Component)]
pub enum Phase {
    #[default]
    InGame,
    ProvidingBonus,
    GameOver,
    Victory,
}

#[derive(Clone, Debug)]
pub enum Bonus {
    AttackPlus { enemy_element: Element },
    DefensePlus { enemy_element: Element },
    DoubleDown,
    Augment(Augmentation),
    Enchant(Aspect),
}

impl Bonus {
    pub fn new_random(bonus_element: Element, rng: &mut impl RngCore) -> Self {
        let random_percent = rand::seq::index::sample(rng, 100, 1).index(0);
        match random_percent {
            0..45 => Bonus::AttackPlus {
                enemy_element: Element::random(rng),
            },
            45..60 => Bonus::DefensePlus {
                enemy_element: Element::random(rng),
            },
            60..65 => Bonus::Augment(Augmentation::Armored),
            65..70 => Bonus::Augment(Augmentation::Combo),
            70..75 => Bonus::Augment(Augmentation::Parry),
            75..90 => Bonus::Enchant(Aspect::new(Element::random_without(rng, bonus_element))),
            _ => Bonus::DoubleDown,
        }
    }

    pub fn get_readable_name(&self) -> String {
        match self {
            Bonus::AttackPlus { enemy_element } => format!("Attack+ vs. {enemy_element}"),
            Bonus::DefensePlus { enemy_element } => format!("Defense+ vs. {enemy_element}"),
            Bonus::DoubleDown => "Double Down".to_string(),
            Bonus::Augment(augmentation) => format!("Augmentation: {augmentation}"),
            Bonus::Enchant(aspect) => format!("Enchantment: Aspect of {}", **aspect),
        }
    }

    pub fn get_description(&self, element: Element) -> String {
        match self {
            Bonus::AttackPlus { enemy_element } => {
                format!("{element} deals 1 additional damage against {enemy_element}")
            }
            Bonus::DefensePlus { enemy_element } => format!(
                "{element} receives 1 less damage (or heals 1 more HP) against {enemy_element}"
            ),
            Bonus::DoubleDown => format!("Doubles all payouts for {element}."),
            Bonus::Augment(augmentation) => augmentation.get_description(element),
            Bonus::Enchant(aspect) => aspect.get_description(element),
        }
    }

    pub fn update_game(
        &self,
        table: &mut ElementTable,
        player_elements: &mut PlayerElements,
        element_to_upgrade: Element,
    ) {
        if !player_elements.elements.contains(&element_to_upgrade) {
            return;
        }
        match self {
            Bonus::AttackPlus { enemy_element } => {
                table.update(
                    element_to_upgrade,
                    *enemy_element,
                    Payout {
                        damage_to_enemy: 1,
                        damage_to_me: 0,
                    },
                );
            }
            Bonus::DefensePlus { enemy_element } => {
                table.update(
                    element_to_upgrade,
                    *enemy_element,
                    Payout {
                        damage_to_enemy: 0,
                        damage_to_me: -1,
                    },
                );
            }
            Bonus::DoubleDown => {
                for enemy_element in Element::ALL {
                    table.double(element_to_upgrade, enemy_element);
                }
            }
            Bonus::Augment(augmentation) => {
                player_elements.augment(element_to_upgrade, *augmentation);
            }
            Bonus::Enchant(aspect) => {
                player_elements.enchant(element_to_upgrade, *aspect);
            }
        }
    }
}

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct Upgrades {
    pub bonuses: Vec<(Bonus, Element)>,
    pub evolutions: BTreeSet<Element>,
}
