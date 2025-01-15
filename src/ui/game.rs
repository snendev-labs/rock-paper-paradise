use std::collections::BTreeSet;

use bevy::{
    ecs::system::StaticSystemParam,
    prelude::{
        AlignItems, App, BackgroundColor, BorderColor, BorderRadius, BuildChildren, Changed,
        ChildBuild, ChildBuilder, Click, Color, Commands, Component, DespawnRecursiveExt, Entity,
        FlexDirection, FlexWrap, IntoSystemConfigs, JustifyContent, Name, Node, Out, Over, Plugin,
        Pointer, PositionType, Query, RemovedComponents, Res, Single, Text, TextFont, Trigger,
        UiRect, Update, Val, With, Without,
    },
};

use crate::{
    Action, Element, ElementTable, Game, Guess, Health, LastOutcome, PlayerElements, Round,
};

use super::{image_bundle, text_bundle, LocalPlayer, RpsGlyphs, UIComponent};

pub struct GameUIPlugin;

impl Plugin for GameUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                Self::track_player_hp,
                Self::track_enemy_hp,
                Self::cleanup_last_outcome_ui,
                Self::track_last_outcome,
            )
                .chain(),
        );
    }
}

impl GameUIPlugin {
    fn track_player_hp(
        player_hp_ui: Option<Single<&mut Text, With<PlayerHPNode>>>,
        player_hp: Option<Single<&Health, With<LocalPlayer>>>,
    ) {
        let Some(mut text) = player_hp_ui else {
            return;
        };
        let Some(hp) = player_hp else {
            return;
        };
        text.0 = PlayerHPNode::text(&hp);
    }

    fn track_enemy_hp(
        enemy_hp_ui: Option<Single<&mut Text, With<EnemyHPNode>>>,
        enemy_hp: Option<Single<&Health, Without<LocalPlayer>>>,
    ) {
        let Some(mut text) = enemy_hp_ui else {
            return;
        };
        let Some(hp) = enemy_hp else {
            return;
        };
        text.0 = EnemyHPNode::text(&hp);
    }

    fn track_last_outcome(
        mut commands: Commands,
        last_outcome_ui: Option<Single<Entity, With<LastOutcomePanel>>>,
        updated_game: Option<Single<&LastOutcome, (With<Game>, Changed<LastOutcome>)>>,
        glyphs: Res<RpsGlyphs>,
    ) {
        let Some(ui) = last_outcome_ui else {
            return;
        };
        let Some(game) = updated_game else {
            return;
        };
        let last_outcome = *game;
        commands
            .entity(*ui)
            .despawn_descendants()
            .with_children(|builder| {
                builder
                    .spawn(LastOutcomePanel::elements_row())
                    .with_child(text_bundle(
                        format!("{}", -last_outcome.0.p1_outcome.damage),
                        TextFont::default(),
                        Node {
                            ..Default::default()
                        },
                    ))
                    .with_child(image_bundle(
                        glyphs.get_image(&last_outcome.0.p1_action).unwrap().clone(),
                        Node {
                            height: Val::Px(100.),
                            margin: UiRect::right(Val::Px(12.)),
                            ..Default::default()
                        },
                    ))
                    .with_child(image_bundle(
                        glyphs.get_image(&last_outcome.0.p2_action).unwrap().clone(),
                        Node {
                            height: Val::Px(100.),
                            margin: UiRect::left(Val::Px(12.)),
                            ..Default::default()
                        },
                    ))
                    .with_child(text_bundle(
                        format!("{}", -last_outcome.0.p2_outcome.damage),
                        TextFont::default(),
                        Node {
                            ..Default::default()
                        },
                    ));
            });
    }

    fn cleanup_last_outcome_ui(
        mut commands: Commands,
        mut removed_outcome: RemovedComponents<LastOutcome>,
        last_outcome_ui: Option<Single<Entity, With<LastOutcomePanel>>>,
    ) {
        let Some(ui) = last_outcome_ui else {
            return;
        };
        let Some(_) = removed_outcome.read().next() else {
            return;
        };
        commands.entity(*ui).try_despawn_recursive();
    }
}

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct GameUIComponent {
    pub player_elements: PlayerElements,
    pub enemy_elements: PlayerElements,
    pub table: ElementTable,
    pub round: Round,
    pub player: Entity,
}

impl UIComponent for GameUIComponent {
    type Params = Res<'static, RpsGlyphs>;

    fn build_ui(self, builder: &mut ChildBuilder<'_>, params: &StaticSystemParam<Self::Params>) {
        builder
            .spawn((EnemyElementsPanel, EnemyElementsPanel::node()))
            .with_children(|builder| {
                for element in &self.enemy_elements.elements {
                    let element = *element;
                    let augmentation = self.enemy_elements.get_augmentation(element);
                    let aspect = self.enemy_elements.get_enchantment(element);

                    builder.spawn((
                        GameButton,
                        GameButton::radius(),
                        GameButton::background_color(),
                        super::image_bundle(
                            params
                                .get_image(&Action::new(
                                    element,
                                    aspect.cloned(),
                                    augmentation.cloned(),
                                ))
                                .unwrap()
                                .clone(),
                            GameButton::node(),
                        ),
                    ));
                }
            });

        builder
            .spawn((GameButtonsPanel, GameButtonsPanel::node()))
            .with_children(|builder| {
                for element in &self.player_elements.elements {
                    let element = *element;
                    let augmentation = self.player_elements.get_augmentation(element);
                    let aspect = self.player_elements.get_enchantment(element);

                    let mut button = builder.spawn((
                        GameButton,
                        GameButton::radius(),
                        GameButton::background_color(),
                        super::image_bundle(
                            params
                                .get_image(&Action::new(
                                    element,
                                    aspect.cloned(),
                                    augmentation.cloned(),
                                ))
                                .unwrap()
                                .clone(),
                            GameButton::node(),
                        ),
                    ));
                    button
                        .observe(GameButton::make_on_click(self.player, element))
                        .observe(GameButton::make_on_over(
                            element,
                            &self.enemy_elements.elements,
                            &self.table,
                        ))
                        .observe(GameButton::make_on_out());
                }
            });

        builder
            .spawn((
                PlayerStatsPanel,
                PlayerStatsPanel::node(),
                PlayerStatsPanel::border_color(),
            ))
            .with_children(|builder| {
                builder.spawn(text_bundle(
                    "Player",
                    Default::default(),
                    Default::default(),
                ));
                builder.spawn((
                    PlayerHPNode,
                    text_bundle("HP: ", Default::default(), Default::default()),
                ));
            });
        builder
            .spawn((
                EnemyStatsPanel,
                EnemyStatsPanel::node(),
                EnemyStatsPanel::border_color(),
            ))
            .with_children(|builder| {
                builder.spawn(text_bundle(
                    format!("Enemy (Round {})", *self.round),
                    Default::default(),
                    Default::default(),
                ));
                builder.spawn((
                    EnemyHPNode,
                    text_bundle("HP:", Default::default(), Default::default()),
                ));
            });

        builder.spawn((LastOutcomePanel, LastOutcomePanel::node()));
    }
}

#[derive(Component)]
struct GameButtonsPanel;

impl GameButtonsPanel {
    fn node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Auto,
            left: Val::Px(0.),
            right: Val::Px(0.),
            bottom: Val::Percent(10.),
            height: Val::Px(120.),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(20.),
            row_gap: Val::Px(20.),
            ..Default::default()
        }
    }
}

#[derive(Component)]
struct EnemyElementsPanel;

impl EnemyElementsPanel {
    fn node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(10.),
            left: Val::Px(0.),
            right: Val::Px(0.),
            bottom: Val::Auto,
            height: Val::Px(120.),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(20.),
            row_gap: Val::Px(20.),
            ..Default::default()
        }
    }
}

#[derive(Component)]
struct GameButton;

impl GameButton {
    fn node() -> Node {
        Node {
            width: Val::Px(120.),
            height: Val::Percent(100.),
            flex_grow: 0.,
            flex_shrink: 0.,
            border: UiRect::all(Val::Px(4.)),
            ..Default::default()
        }
    }

    fn radius() -> BorderRadius {
        BorderRadius::all(Val::Px(16.))
    }

    fn background_color() -> BackgroundColor {
        Color::hsla(0., 0., 0.08, 0.5).into()
    }

    fn make_on_click(
        player: Entity,
        guess: Element,
    ) -> impl FnMut(Trigger<Pointer<Click>>, Commands) {
        move |_trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
            commands.entity(player).insert(Guess::new(guess));
        }
    }

    fn make_on_over(
        element: Element,
        enemy_elements: &BTreeSet<Element>,
        table: &ElementTable,
    ) -> impl FnMut(Trigger<Pointer<Over>>, Commands) {
        let payouts_description = enemy_elements
            .iter()
            .map(|enemy_element| {
                let payout = table.evaluate(element, *enemy_element);
                format!(
                    "{element} vs. {enemy_element}: deal {}, take {}",
                    payout.damage_to_enemy, payout.damage_to_me,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        move |trigger: Trigger<Pointer<Over>>, mut commands: Commands| {
            let tooltip_ui = commands
                .spawn((
                    Name::new("Payout Tooltip"),
                    PayoutTooltip {
                        element_node: trigger.entity(),
                    },
                    super::text_bundle(
                        payouts_description.clone(),
                        TextFont::default(),
                        PayoutTooltip::node(),
                    ),
                    PayoutTooltip::border_color(),
                ))
                .id();
            commands.entity(trigger.entity()).add_child(tooltip_ui);
        }
    }

    #[allow(clippy::type_complexity)]
    fn make_on_out() -> impl FnMut(Trigger<Pointer<Out>>, Commands, Query<(Entity, &PayoutTooltip)>)
    {
        move |trigger: Trigger<Pointer<Out>>,
              mut commands: Commands,
              tooltips: Query<(Entity, &PayoutTooltip)>| {
            for (panel, PayoutTooltip { element_node }) in &tooltips {
                if *element_node == trigger.entity() {
                    commands.entity(panel).despawn_recursive();
                }
            }
        }
    }
}

#[derive(Component)]
struct PlayerStatsPanel;

impl PlayerStatsPanel {
    fn node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Auto,
            left: Val::Px(100.),
            right: Val::Auto,
            bottom: Val::Percent(20.),
            height: Val::Px(120.),
            width: Val::Px(120.),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(20.),
            row_gap: Val::Px(20.),
            border: UiRect::all(Val::Px(2.)),
            ..Default::default()
        }
    }

    fn border_color() -> BorderColor {
        BorderColor(Color::BLACK)
    }
}

#[derive(Component)]
struct EnemyStatsPanel;

impl EnemyStatsPanel {
    fn node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(20.),
            left: Val::Auto,
            right: Val::Px(100.),
            bottom: Val::Auto,
            height: Val::Px(120.),
            width: Val::Px(120.),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(20.),
            row_gap: Val::Px(20.),
            border: UiRect::all(Val::Px(2.)),
            ..Default::default()
        }
    }

    fn border_color() -> BorderColor {
        BorderColor(Color::BLACK)
    }
}

#[derive(Component)]
pub struct PlayerHPNode;

impl PlayerHPNode {
    fn text(health: &Health) -> String {
        format!("HP: {}", **health)
    }
}

#[derive(Component)]
pub struct EnemyHPNode;

impl EnemyHPNode {
    fn text(health: &Health) -> String {
        format!("HP: {}", **health)
    }
}

#[derive(Component)]
pub struct PayoutTooltip {
    pub element_node: Entity,
}

impl PayoutTooltip {
    fn node() -> Node {
        Node {
            bottom: Val::Px(250.),
            width: Val::Px(600.),
            height: Val::Percent(150.),
            flex_grow: 0.,
            flex_shrink: 0.,
            border: UiRect::all(Val::Px(2.)),
            ..Default::default()
        }
    }

    fn border_color() -> BorderColor {
        BorderColor(Color::BLACK)
    }
}

#[derive(Component)]
struct LastOutcomePanel;

impl LastOutcomePanel {
    fn node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(30.),
            left: Val::Percent(30.),
            right: Val::Percent(30.),
            bottom: Val::Percent(30.),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.),
            ..Default::default()
        }
    }

    pub fn elements_row() -> Node {
        Node {
            width: Val::Percent(100.),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.),
            ..Default::default()
        }
    }
}
