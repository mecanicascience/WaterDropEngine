use bevy::prelude::*;
use wde_render::components::{Camera, CameraController, CameraView};

use crate::{components::mesh_component::MeshComponent, features::mesh_feature::MeshFeature};

/// Start the game engine plugin
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, init)
            .add_plugins(MeshComponent)
            .add_plugins(MeshFeature);
    }
}

fn init(mut commands: Commands) {
    // Creates a camera
    commands.spawn(
        (Camera {
            transform: Transform {
                translation: Vec3::new(2.0, 2.0, 2.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::splat(1.0),
            }.looking_at(Vec3::ZERO, Vec3::Y),
            view: CameraView::default()
        },
        CameraController::default()
    ));
}
