#![allow(clippy::just_underscores_and_digits)]

use bevy::{input::InputPlugin, log::{Level, LogPlugin}, prelude::*};
use game::GamePlugin;

mod game;
mod renderer;
mod scene;

pub fn start_game() {
    // Log level
    #[cfg(debug_assertions)]
    let level = if cfg!(feature = "trace") {
        Level::TRACE
    } else {
        Level::DEBUG
    };
    #[cfg(not(debug_assertions))]
    let level = Level::INFO;

    // Create the app
    let mut app = App::new();

    // Add default bevy plugins
    let app = app
        .add_plugins(MinimalPlugins)
        .add_plugins(LogPlugin {
            level,
            filter: "wgpu_hal=warn,wgpu_core=warn".to_string(),
            custom_layer: |_| None,
        })
        .add_plugins(HierarchyPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(AssetPlugin {
            mode: AssetMode::Unprocessed,
            file_path: "res".to_string(),
            ..Default::default()
        });

    // Add the game plugin
    let app = app.add_plugins(GamePlugin);

    // Run the app
    app.run();
}
