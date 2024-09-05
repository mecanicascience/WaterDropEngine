#![allow(clippy::just_underscores_and_digits)]

use bevy::{input::InputPlugin, log::{Level, LogPlugin}, prelude::*};
use test_render::{test_component::TestComponentPlugin, test_feature::TestFeature};
use wde_render::RenderPlugin;

mod test_render;

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
            filter: "wgpu_hal=warn,wgpu_core=warn,naga=warn".to_string(),
            custom_layer: |_| None,
        })
        .add_plugins(HierarchyPlugin)
        .add_plugins(InputPlugin)
        .add_plugins(AssetPlugin {
            mode: AssetMode::Unprocessed,
            file_path: "res".to_string(),
            ..Default::default()
        });
    info!("Starting game engine.");

    // Add the render plugin
    app.add_plugins(RenderPlugin);

    // Add the game plugin
    app
        .add_plugins(TestComponentPlugin)
        .add_plugins(TestFeature);

    // Run the app
    info!("Running game engine.");
    app.run();
}
