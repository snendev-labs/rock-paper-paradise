#[allow(unused)]
use bevy::{
    asset::AssetMetaCheck,
    prelude::{App, AssetPlugin, PluginGroup, Window, WindowPlugin},
    window::WindowResolution,
    DefaultPlugins,
};

use rock_paper_paradise::RockPaperParadisePlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // resolution: WindowResolution::new(430., 932.),
                    canvas: Some("#game-canvas".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..Default::default()
            }),
    );
    app.add_plugins(RockPaperParadisePlugins);
    app.run();
}
