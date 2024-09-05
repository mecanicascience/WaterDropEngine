#![allow(clippy::just_underscores_and_digits)]

pub mod assets;
pub mod pipelines;
pub mod components;
pub mod core;
pub mod features;

use core::RenderCorePlugin;

use assets::SceneResourcesPlugin;
use bevy::{app::{App, Plugin, Startup}, log::info, math::{Quat, Vec3}, prelude::Commands};
use components::{CameraComponent, CameraViewComponent, TransformComponent};

pub struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        // First, add the renderer plugin
        app.add_plugins(RenderCorePlugin);

        // Register the scene plugin
        app.add_plugins(SceneResourcesPlugin);

        // Setup app
        app
            .add_systems(Startup, init);
    }

    fn finish(&self, _app: &mut App) {
        info!("Render plugin initialized");
    }
}

fn init(mut commands: Commands) {
    // Creates a camera
    commands.spawn(CameraComponent {
        transform: TransformComponent {
            position: Vec3::new(0.0, 0.0, 5.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(1.0),
        },
        view: CameraViewComponent {
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
        },
    });
}
