use bevy::prelude::*;
use wde_render::components::{CameraComponent, CameraViewComponent, TransformComponent};

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
    commands.spawn(CameraComponent {
        transform: TransformComponent {
            position: Vec3::new(0.0, 0.0, 1.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(1.0),
        },
        view: CameraViewComponent::default()
    });
}