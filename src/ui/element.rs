use bevy::prelude::{
    BackgroundColor, BuildChildren, Commands, Component, DespawnRecursiveExt, Entity, Name, Node,
    Out, Over, Pointer, PositionType, Query, TextFont, Trigger, Val,
};

use crate::Element;

#[derive(Component)]
pub struct ElementTooltip {
    pub node: Entity,
}

impl ElementTooltip {
    pub fn node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Percent(100.),
            ..Default::default()
        }
    }

    pub fn background_color() -> BackgroundColor {
        use bevy::color::palettes::css;
        css::DARK_SLATE_BLUE.into()
    }

    pub fn make_on_over(element: Element) -> impl FnMut(Trigger<Pointer<Over>>, Commands) {
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
    pub fn make_on_out(
    ) -> impl FnMut(Trigger<Pointer<Out>>, Commands, Query<(Entity, &ElementTooltip)>) {
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
