use bevy::app::{PluginGroup, PluginGroupBuilder};

mod campaign;
pub use campaign::*;

mod game;
pub use game::*;

mod player;
pub use player::*;

mod rps;
pub use rps::*;

mod ui;

pub struct RockPaperParadisePlugins;

impl PluginGroup for RockPaperParadisePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(RpsPlugin)
            .add(CampaignPlugin)
            .add(GamePlugin)
            .add(ui::UIPlugin)
    }
}
