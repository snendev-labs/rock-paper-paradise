use derive_more::derive::Deref;

use bevy::prelude::{
    App, Commands, Component, Entity, Event, IntoSystemConfigs, Name, Plugin, Query, SystemSet,
    Trigger, Update, With, Without,
};

use crate::{Action, Combo, ElementTable, Guess, Health, Outcome, Player, PlayerElements, Stagger};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(SpawnGame::observer);
        app.add_systems(
            Update,
            (Self::throw_hands, Self::detect_game_over)
                .chain()
                .in_set(GameSystems),
        );
    }
}

impl GamePlugin {
    #[allow(clippy::type_complexity)]
    fn throw_hands(
        mut commands: Commands,
        games: Query<(Entity, &Game, &ElementTable)>,
        mut players: Query<
            (
                &mut Health,
                &PlayerElements,
                &Guess,
                Option<&Combo>,
                Option<&Stagger>,
            ),
            With<Player>,
        >,
    ) {
        for (game_entity, game, table) in &games {
            let Ok(
                [(mut p1_hp, p1_elements, p1_guess, p1_combo, p1_stagger), (mut p2_hp, p2_elements, p2_guess, p2_combo, p2_stagger)],
            ) = players.get_many_mut([game.player_one, game.player_two])
            else {
                continue;
            };

            let p1_enchantment = p1_elements.get_enchantment(**p1_guess);
            let p2_enchantment = p2_elements.get_enchantment(**p2_guess);
            let p1_augmentation = p1_elements.get_augmentation(**p1_guess);
            let p2_augmentation = p2_elements.get_augmentation(**p2_guess);
            let p1_action = Action::new(
                **p1_guess,
                p1_enchantment.cloned(),
                p1_augmentation.cloned(),
            );
            let p2_action = Action::new(
                **p2_guess,
                p2_enchantment.cloned(),
                p2_augmentation.cloned(),
            );

            let outcome = Outcome::new(
                table, p1_action, p1_combo, p1_stagger, p2_action, p2_combo, p2_stagger,
            );

            bevy::log::info!(
                "{} vs {} =>  I'm hurt {} & enemy hurt {}",
                outcome.p1_action.guess,
                outcome.p2_action.guess,
                outcome.p1_outcome.damage,
                outcome.p2_outcome.damage,
            );
            **p1_hp = p1_hp.saturating_add_signed(-outcome.p1_outcome.damage);
            **p2_hp = p2_hp.saturating_add_signed(-outcome.p2_outcome.damage);
            commands.entity(game.player_one).remove::<Guess>();
            commands.entity(game.player_two).remove::<Guess>();
            if let Some(Combo) = outcome.p1_outcome.next_combo {
                commands.entity(game.player_one).insert(Combo);
            }
            if let Some(Combo) = outcome.p2_outcome.next_combo {
                commands.entity(game.player_two).insert(Combo);
            }
            if let Some(Stagger) = outcome.p1_outcome.next_stagger {
                commands.entity(game.player_one).insert(p1_guess.clone());
            }
            if let Some(Stagger) = outcome.p2_outcome.next_stagger {
                commands.entity(game.player_two).insert(p2_guess.clone());
            }
            commands.entity(game_entity).insert(LastOutcome(outcome));
        }
    }

    fn detect_game_over(
        mut commands: Commands,
        games: Query<(Entity, &Game), Without<GameOver>>,
        players: Query<&Health, With<Player>>,
    ) {
        for (
            game,
            Game {
                player_one,
                player_two,
            },
        ) in &games
        {
            let Ok([p1_hp, p2_hp]) = players.get_many([*player_one, *player_two]) else {
                continue;
            };
            let game_over = match (**p1_hp, **p2_hp) {
                (0, 0) => GameOver::Draw,
                (0, _) => GameOver::Winner(*player_two),
                (_, 0) => GameOver::Winner(*player_one),
                (_, _) => {
                    continue;
                }
            };
            commands.entity(game).insert(game_over);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub struct GameSystems;

#[derive(Default)]
#[derive(Event)]
pub struct SpawnGame {
    pub game: Option<Entity>,
    pub player_one: Option<Entity>,
    pub player_two: Option<Entity>,
    pub table: Option<ElementTable>,
}

impl SpawnGame {
    fn observer(trigger: Trigger<Self>, mut commands: Commands) {
        let game = trigger
            .event()
            .game
            .unwrap_or_else(|| commands.spawn_empty().id());
        let player_one = trigger
            .event()
            .player_one
            .unwrap_or_else(|| commands.spawn_empty().id());
        let player_two = trigger
            .event()
            .player_two
            .unwrap_or_else(|| commands.spawn_empty().id());

        commands.entity(game).insert(Game {
            player_one,
            player_two,
        });
        if let Some(table) = &trigger.event().table {
            commands.entity(game).insert(table.clone());
        }
        commands.entity(player_one).insert((Player, InGame(game)));
        commands.entity(player_two).insert((Player, InGame(game)));
    }
}

#[derive(Component)]
#[require(ElementTable, Name(|| Name::new("Game")))]
pub struct Game {
    pub player_one: Entity,
    pub player_two: Entity,
}

#[derive(Deref)]
#[derive(Component)]
pub struct InGame(pub Entity);

#[derive(Component)]
pub struct LastOutcome(pub Outcome);

#[derive(Component)]
pub enum GameOver {
    Winner(Entity),
    Draw,
}
