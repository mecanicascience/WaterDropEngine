use mc_compute_core::MarchingCubesComputePass;
use mc_render_core::MarchingCubesRenderPass;

use bevy::prelude::*;

pub mod mc_compute_core;
pub mod mc_compute_pipeline;
pub mod mc_render_core;
pub mod mc_render_pipeline;

pub struct TerrainFeaturesPlugin;
impl Plugin for TerrainFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MarchingCubesComputePass)
            .add_plugins(MarchingCubesRenderPass);
    }
}