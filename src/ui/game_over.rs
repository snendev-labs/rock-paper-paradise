use bevy::{
    color::Color,
    ecs::system::StaticSystemParam,
    prelude::{
        BuildChildren, ChildBuild, ChildBuilder, Click, Commands, Component, Entity, Node, Pointer,
        TextFont, Trigger, Val,
    },
    ui::{BorderColor, FlexDirection, JustifyContent, PositionType},
};

use crate::{Level, Round, SpawnCampaign};

use super::{text_bundle, UIComponent};

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct GameOverUIComponent {
    is_victory: bool,
    round: Round,
    level: Level,
    campaign: Entity,
    player: Entity,
}

impl GameOverUIComponent {
    pub fn victory(campaign: Entity, player: Entity, round: Round, level: Level) -> Self {
        Self {
            is_victory: true,
            campaign,
            player,
            round,
            level,
        }
    }

    pub fn loss(campaign: Entity, player: Entity, round: Round, level: Level) -> Self {
        Self {
            is_victory: false,
            campaign,
            player,
            round,
            level,
        }
    }
}

impl UIComponent for GameOverUIComponent {
    type Params = ();

    fn build_ui(self, builder: &mut ChildBuilder<'_>, _: &StaticSystemParam<Self::Params>) {
        builder
            .spawn((GameOverUIPanel, GameOverUIPanel::outer_node()))
            .with_children(|builder| {
                builder
                    .spawn((GameOverUIPanel::inner_node(), BorderColor(Color::BLACK)))
                    .with_children(|builder| {
                        builder.spawn(text_bundle(
                            if self.is_victory {
                                format!(
                                    "You win! You defeated the game in {:} battles.",
                                    *self.round
                                )
                            } else {
                                format!(
                                    "You survived {:} battles until losing at level {:?}",
                                    *self.round, self.level
                                )
                            },
                            TextFont::default(),
                            Node::default(),
                        ));
                        let handle_click =
                            move |_trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
                                commands.entity(self.campaign).despawn();
                                commands.trigger(SpawnCampaign {
                                    player: Some(self.player),
                                });
                            };
                        builder
                            .spawn(text_bundle(
                                "Try again",
                                TextFont::default(),
                                Node::default(),
                            ))
                            .observe(handle_click);
                    });
            });
    }
}

#[derive(Component)]
pub struct GameOverUIPanel;

impl GameOverUIPanel {
    fn outer_node() -> Node {
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(50.),
            left: Val::Percent(50.),
            right: Val::Percent(50.),
            bottom: Val::Percent(50.),
            ..Default::default()
        }
    }

    fn inner_node() -> Node {
        Node {
            position_type: PositionType::Relative,
            top: Val::Px(-100.),
            left: Val::Px(-150.),
            width: Val::Px(200.),
            height: Val::Px(300.),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            ..Default::default()
        }
    }
}
