use bevy::prelude::*;
use wde_render::components::{Camera, CameraController, CameraView};

use crate::components::mesh_component::MeshComponent;

/// Start the game engine plugin
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, init)
            .add_plugins(MeshComponent);
    }
}

fn init(mut commands: Commands) {
    // Creates a camera
    commands.spawn(
        (Camera {
            transform: Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            view: CameraView::default()
        },
        CameraController::default()
    ));
}
