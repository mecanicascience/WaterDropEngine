use bevy::prelude::*;
use crate::{scene::components::{CameraComponent, CameraViewComponent, TransformComponent}, renderer::RenderPlugin};

use crate::scene::ScenePlugin;

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // First, add the renderer plugin
        app.add_plugins(RenderPlugin);

        // Register plugins
        app.add_plugins(ScenePlugin);

        // Setup app
        app
            .add_systems(Startup, init);
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
            // fov: 60.0,
            // near: 0.1,
            // far: 1000.0,
        },
    });
}
