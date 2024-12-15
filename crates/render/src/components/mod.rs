use bevy::prelude::*;

mod transform;
mod camera;
mod camera_controller;
mod lights;
mod terrain;

pub use transform::*;
pub use camera::*;
pub use camera_controller::*;
pub use lights::*;
pub use terrain::*;

pub struct RenderComponentsPlugin;
impl Plugin for RenderComponentsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(CameraControllerPlugin)
            .add_plugins(ChunkSpawnerPlugin);
    }
}

