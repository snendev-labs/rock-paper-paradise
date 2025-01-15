use bevy::{
    ecs::system::StaticSystemParam,
    prelude::{
        ChildBuild, ChildBuilder, Click, Commands, Component, Node, Pointer, TextFont, Trigger, Val,
    },
};

use crate::SpawnCampaign;

use super::UIComponent;

#[derive(Clone, Debug)]
#[derive(Component)]
pub struct HomeMenuUIComponent;

impl UIComponent for HomeMenuUIComponent {
    type Params = ();

    fn build_ui(self, builder: &mut ChildBuilder<'_>, _: &StaticSystemParam<Self::Params>) {
        builder
            .spawn(super::text_bundle(
                "Play",
                TextFont::default(),
                Node {
                    width: Val::Px(140.),
                    height: Val::Percent(100.),
                    left: Val::Percent(50.),
                    top: Val::Percent(50.),
                    flex_grow: 0.,
                    flex_shrink: 0.,
                    ..Default::default()
                },
            ))
            .observe(|trigger: Trigger<Pointer<Click>>, mut commands: Commands| {
                let local_player = commands.spawn(super::LocalPlayer).id();
                bevy::log::info!("HEllo...?");
                commands.trigger(SpawnCampaign {
                    player: Some(local_player),
                });
                commands.entity(trigger.entity()).despawn();
            });
    }
}
