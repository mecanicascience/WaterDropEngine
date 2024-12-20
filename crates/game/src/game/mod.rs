pub mod scene;
pub mod mc_compute_core;
pub mod mc_compute_pipeline;
pub mod mc_render_core;
pub mod mc_render_pipeline;

use bevy::prelude::*;
use mc_compute_core::MarchingCubesPluginCompute;
use mc_render_core::MarchingCubesPluginRender;
use wde_render::{components::{ActiveCamera, Camera, CameraController, CameraView, DirectionalLight}, utils::Color};

pub struct MarchingCubesPlugin;
impl Plugin for MarchingCubesPlugin {
    fn build(&self, app: &mut App) {
        // Add the plugins
        app
            .add_plugins(MarchingCubesPluginCompute)
            .add_plugins(MarchingCubesPluginRender);

        // Add the systems
        app.add_systems(Startup, init);
    }
}

fn init(mut commands: Commands) {
    // Main camera
    commands.spawn(
        (Camera {
            transform: Transform::from_xyz(5.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            view: CameraView::default()
        },
        CameraController::default(),
        ActiveCamera
    ));
    
    // Light
    commands.spawn(DirectionalLight {
        direction: Vec3::new(0.1, -0.8, 0.2),
        ambient: Color::from_srgba(0.3, 0.3, 0.3, 1.0),
        ..Default::default()
    });
    commands.spawn(DirectionalLight {
        direction: Vec3::new(-0.3, 0.8, -0.5),
        ambient: Color::from_srgba(0.1, 0.1, 0.1, 1.0),
        ..Default::default()
    });
}
