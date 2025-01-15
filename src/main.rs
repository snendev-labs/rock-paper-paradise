#[allow(unused)]
use bevy::{
    prelude::{App, PluginGroup, Window, WindowPlugin},
    window::WindowResolution,
    DefaultPlugins,
};

use rock_paper_paradise::RockPaperParadisePlugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            // resolution: WindowResolution::new(430., 932.),
            ..Default::default()
        }),
        ..Default::default()
    }));
    app.add_plugins(RockPaperParadisePlugins);
    app.run();
}
