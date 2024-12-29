use bevy::prelude::*;
use generate::MCGeneratePlugin;
use mc_compute_main::{MCComputeHandler, MCComputeHandlerGPU};
use process::MCProcessPlugin;
use render::MCRenderPlugin;
use spawn::MCSpawnPlugin;
use wde_render::core::{Extract, RenderApp};

mod mc_chunk;
mod mc_compute_main;
mod spawn;
mod generate;
mod process;
mod render;

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // Add the different generation plugins
        app
            .add_plugins(MCSpawnPlugin)
            .add_plugins(MCGeneratePlugin)
            .add_plugins(MCProcessPlugin)
            .add_plugins(MCRenderPlugin);

        // Add the compute main plugin
        app
            .init_resource::<MCComputeHandler>()
            .add_systems(Startup, MCComputeHandler::init);
        app.get_sub_app_mut(RenderApp).unwrap()
            .init_resource::<MCComputeHandlerGPU>()
            .add_systems(Extract, MCComputeHandler::extract);
    }
}
