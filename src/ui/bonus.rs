use bevy::{
    ecs::system::StaticSystemParam,
    prelude::{
        AlignItems, BackgroundColor, BorderColor, BuildChildren, ChildBuild, ChildBuilder, Click,
        Color, Commands, Component, DespawnRecursiveExt, Entity, FlexDirection, FlexWrap,
        JustifyContent, Name, Node, Out, Over, Pointer, PositionType, Query, Res, Single, TextFont,
        Trigger, UiRect, Val, With,
    },
};

use crate::{Bonus, Campaign, Element, ElementTable, Level, Phase, PlayerElements, Upgrades};

use super::{image_bundle, text_bundle, LocalPlayer, RpsGlyphs, UIComponent};

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct BonusUIComponent {
    upgrades: Upgrades,
    table: ElementTable,
}

impl BonusUIComponent {
    pub fn new(table: ElementTable, upgrades: Upgrades) -> Self {
        Self { table, upgrades }
    }
}

impl UIComponent for BonusUIComponent {
    type Params = Res<'static, RpsGlyphs>;

    fn build_ui(self, builder: &mut ChildBuilder<'_>, params: &StaticSystemParam<Self::Params>) {
        builder
            .spawn((ElementsTablePanel, ElementsTablePanel::container()))
            .with_children(|builder| {
                builder
                    .spawn((
                        ElementsTablePanel::table(),
                        BackgroundColor(Color::hsla(0., 0., 0.2, 0.9)),
                        BorderColor(Color::BLACK),
                    ))
                    .with_children(|builder| {
                        // header row
                        builder
                            .spawn(ElementsTablePanel::row())
                            .with_children(|builder| {
                                // top left corner cell
                                builder
                                    .spawn((ElementsTablePanel::cell(), BorderColor(Color::BLACK)));
                                // top header cells
                                for enemy_element in Element::ALL {
                                    builder
                                        .spawn((
                                            ElementsTablePanel::cell(),
                                            BorderColor(Color::BLACK),
                                        ))
                                        .with_child(image_bundle(
                                            params
                                                .get_image(&enemy_element.into())
                                                .unwrap()
                                                .clone(),
                                            Node {
                                                width: Val::Percent(100.),
                                                height: Val::Percent(100.),
                                                ..Default::default()
                                            },
                                        ))
                                        .observe(ElementTooltip::make_on_over(enemy_element))
                                        .observe(ElementTooltip::make_on_out());
                                }
                            });
                        // data rows
                        for element in Element::ALL {
                            builder
                                .spawn(ElementsTablePanel::row())
                                .with_children(|builder| {
                                    // left header cell
                                    builder
                                        .spawn((
                                            ElementsTablePanel::cell(),
                                            BorderColor(Color::BLACK),
                                        ))
                                        .with_child(image_bundle(
                                            params.get_image(&element.into()).unwrap().clone(),
                                            Node {
                                                width: Val::Percent(100.),
                                                height: Val::Percent(100.),
                                                ..Default::default()
                                            },
                                        ))
                                        .observe(ElementTooltip::make_on_over(element))
                                        .observe(ElementTooltip::make_on_out());
                                    // data cells
                                    for enemy_element in Element::ALL {
                                        let payout = self.table.evaluate(element, enemy_element);
                                        builder
                                            .spawn((
                                                ElementsTablePanel::cell(),
                                                BorderColor(Color::BLACK),
                                            ))
                                            .with_child(text_bundle(
                                                format!(
                                                    "{}|{}",
                                                    payout.damage_to_me, payout.damage_to_enemy
                                                ),
                                                TextFont::default(),
                                                Node::DEFAULT,
                                            ));
                                    }
                                });
                        }
                    });
            });

        builder
            .spawn((BonusSelectionPanel, BonusSelectionPanel::node()))
            .with_children(|builder| {
                for (bonus, bonus_element) in self.upgrades.bonuses {
                    let bonus_name = bonus.get_readable_name();
                    let bonus_description = bonus.get_description(bonus_element);
                    builder
                        .spawn((
                            BonusButton,
                            super::text_bundle(
                                format!("{bonus_element}: {}", bonus_name.clone()),
                                TextFont::default(),
                                BonusButton::node(),
                            ),
                        ))
                        .observe(BonusButton::make_on_click(bonus, bonus_element))
                        .observe(BonusTooltip::make_on_over(bonus_description))
                        .observe(BonusTooltip::make_on_out());
                }
            });

        builder
            .spawn((EvolutionSelectionPanel, EvolutionSelectionPanel::node()))
            .with_children(|builder| {
                for element in self.upgrades.evolutions {
                    builder
                        .spawn((
                            EvolutionButton,
                            super::image_bundle(
                                params.get_image(&element.into()).unwrap().clone(),
                                EvolutionButton::node(),
                            ),
                        ))
                        .observe(EvolutionButton::make_on_click(element))
                        .observe(ElementTooltip::make_on_over(element))
                        .observe(ElementTooltip::make_on_out());
                }
            });
    }
}

#[derive(Component)]
pub struct ElementsTablePanel;

impl ElementsTablePanel {
    pub fn container() -> Node {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(5.),
            left: Val::Px(0.),
            right: Val::Px(0.),
            bottom: Val::Auto,
            height: Val::Px(402.),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        }
    }

    pub fn table() -> Node {
        Node {
            width: Val::Px(402.),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_wrap: FlexWrap::Wrap,
            border: UiRect::all(Val::Px(1.)),
            ..Default::default()
        }
    }

    pub fn row() -> Node {
        Node {
            height: Val::Px(50.),
            padding: UiRect::all(Val::Px(8.)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        }
    }

    pub fn cell() -> Node {
        Node {
            width: Val::Px(50.),
            height: Val::Px(50.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.)),
            ..Default::default()
        }
    }
}

#[derive(Component)]
pub struct BonusSelectionPanel;

impl BonusSelectionPanel {
    pub fn node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Auto,
            left: Val::Px(100.),
            right: Val::Auto,
            bottom: Val::Percent(20.),
            height: Val::Px(120.),
            width: Val::Px(60.),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(20.),
            row_gap: Val::Px(20.),
            ..Default::default()
        }
    }
}

#[derive(Component)]
pub struct BonusButton;

impl BonusButton {
    pub fn node() -> Node {
        Node {
            width: Val::Px(140.),
            height: Val::Percent(100.),
            flex_grow: 0.,
            flex_shrink: 0.,
            ..Default::default()
        }
    }

    #[allow(clippy::type_complexity)]
    fn make_on_click(
        bonus: Bonus,
        bonus_element: Element,
    ) -> impl FnMut(
        Trigger<Pointer<Click>>,
        Single<(&Campaign, &mut ElementTable, &mut Phase)>,
        Single<&mut PlayerElements, With<LocalPlayer>>,
    ) {
        move |_trigger: Trigger<Pointer<Click>>,
              mut campaign: Single<(&Campaign, &mut ElementTable, &mut Phase)>,
              mut local_player: Single<&mut PlayerElements, With<LocalPlayer>>| {
            bonus.update_game(&mut campaign.1, &mut local_player, bonus_element);
            *campaign.2 = Phase::InGame;
        }
    }
}

#[derive(Component)]
pub struct BonusTooltip {
    pub bonus_node: Entity,
}

impl BonusTooltip {
    pub fn node() -> Node {
        Node {
            bottom: Val::Px(160.),
            width: Val::Px(600.),
            height: Val::Px(100.),
            flex_grow: 0.,
            flex_shrink: 0.,
            border: UiRect::all(Val::Px(1.)),
            ..Default::default()
        }
    }

    pub fn background_color() -> BackgroundColor {
        use bevy::color::palettes::css;
        css::DARK_SLATE_BLUE.into()
    }

    fn make_on_over(bonus_description: String) -> impl FnMut(Trigger<Pointer<Over>>, Commands) {
        move |trigger: Trigger<Pointer<Over>>, mut commands: Commands| {
            let tooltip_ui = commands
                .spawn((
                    Name::new("Bonus Tooltip"),
                    Self {
                        bonus_node: trigger.entity(),
                    },
                    super::text_bundle(
                        bonus_description.clone(),
                        TextFont::default(),
                        Self::node(),
                    ),
                    Self::background_color(),
                ))
                .id();
            commands.entity(trigger.entity()).add_child(tooltip_ui);
        }
    }

    #[allow(clippy::type_complexity)]
    fn make_on_out() -> impl FnMut(Trigger<Pointer<Out>>, Commands, Query<(Entity, &BonusTooltip)>)
    {
        move |trigger: Trigger<Pointer<Out>>,
              mut commands: Commands,
              tooltips: Query<(Entity, &BonusTooltip)>| {
            for (panel, BonusTooltip { bonus_node }) in &tooltips {
                if *bonus_node == trigger.entity() {
                    commands.entity(panel).despawn_recursive();
                }
            }
        }
    }
}

#[derive(Component)]
pub struct EvolutionSelectionPanel;

impl EvolutionSelectionPanel {
    pub fn node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Auto,
            left: Val::Auto,
            right: Val::Px(300.),
            bottom: Val::Percent(20.),
            height: Val::Px(120.),
            width: Val::Px(200.),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(20.),
            row_gap: Val::Px(20.),
            ..Default::default()
        }
    }
}

#[derive(Component)]
pub struct EvolutionButton;

impl EvolutionButton {
    pub fn node() -> Node {
        Node {
            width: Val::Px(140.),
            height: Val::Percent(100.),
            flex_grow: 0.,
            flex_shrink: 0.,
            ..Default::default()
        }
    }

    #[allow(clippy::type_complexity)]
    fn make_on_click(
        element: Element,
    ) -> impl FnMut(
        Trigger<Pointer<Click>>,
        Single<(&mut Phase, &mut Level), With<Campaign>>,
        Single<&mut PlayerElements, With<LocalPlayer>>,
    ) {
        move |_trigger: Trigger<Pointer<Click>>,
              mut campaign: Single<(&mut Phase, &mut Level), With<Campaign>>,
              mut local_player: Single<&mut PlayerElements, With<LocalPlayer>>| {
            local_player.insert(element);
            campaign.1.increment();
            *campaign.0 = Phase::InGame;
        }
    }
}

#[derive(Component)]
pub struct ElementTooltip {
    pub node: Entity,
}

impl ElementTooltip {
    pub fn node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(60.),
            ..Default::default()
        }
    }

    pub fn background_color() -> BackgroundColor {
        use bevy::color::palettes::css;
        css::DARK_SLATE_BLUE.into()
    }

    fn make_on_over(element: Element) -> impl FnMut(Trigger<Pointer<Over>>, Commands) {
        move |trigger: Trigger<Pointer<Over>>, mut commands: Commands| {
            let tooltip_ui = commands
                .spawn((
                    Name::new("Element Tooltip"),
                    Self {
                        node: trigger.entity(),
                    },
                    super::text_bundle(element.to_string(), TextFont::default(), Self::node()),
                    Self::background_color(),
                ))
                .id();
            commands.entity(trigger.entity()).add_child(tooltip_ui);
        }
    }

    #[allow(clippy::type_complexity)]
    fn make_on_out() -> impl FnMut(Trigger<Pointer<Out>>, Commands, Query<(Entity, &ElementTooltip)>)
    {
        move |trigger: Trigger<Pointer<Out>>,
              mut commands: Commands,
              tooltips: Query<(Entity, &ElementTooltip)>| {
            for (panel, ElementTooltip { node }) in &tooltips {
                if *node == trigger.entity() {
                    commands.entity(panel).despawn_recursive();
                }
            }
        }
    }
}
