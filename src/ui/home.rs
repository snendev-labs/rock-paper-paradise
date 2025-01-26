use bevy::{
    ecs::system::StaticSystemParam,
    prelude::{
        AlignItems, AssetServer, BuildChildren, ChildBuild, ChildBuilder, Click, Commands,
        Component, FlexDirection, JustifyContent, Node, Pointer, Res, TextFont, Trigger, UiRect,
        Val,
    },
};

use crate::SpawnCampaign;

use super::UIComponent;

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct HomeMenuUIComponent;

impl UIComponent for HomeMenuUIComponent {
    type Params = Res<'static, AssetServer>;

    fn build_ui(self, builder: &mut ChildBuilder<'_>, params: &StaticSystemParam<Self::Params>) {
        const WIDTH_PX: f32 = 500.;
        const HEIGHT_PX: f32 = 600.;

        builder
            .spawn(Node {
                left: Val::Percent(50.),
                top: Val::Percent(50.),
                width: Val::Px(WIDTH_PX),
                height: Val::Px(HEIGHT_PX),
                margin: UiRect {
                    left: Val::Px(-WIDTH_PX / 2.),
                    top: Val::Px(-HEIGHT_PX / 2.),
                    ..Default::default()
                },
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..Default::default()
            })
            .with_children(|builder| {
                builder.spawn(super::image_bundle(
                    params.load("septagon.png"),
                    Node {
                        width: Val::Px(WIDTH_PX),
                        height: Val::Px(WIDTH_PX),
                        ..Default::default()
                    },
                ));
                builder
                    .spawn(super::text_bundle(
                        "Play",
                        TextFont::default(),
                        Node::default(),
                    ))
                    .observe(|trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
                        let local_player = commands.spawn(super::LocalPlayer).id();
                        commands.trigger(SpawnCampaign {
                            player: Some(local_player),
                        });
                        commands.entity(trigger.entity()).despawn();
                    });
            });
    }
}
