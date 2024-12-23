use mc_compute_core::{GpuMarchingCubesDescription, MarchingCubesComputeTask, MarchingCubesHandler, MarchingCubesHandlerGPU};
use mc_compute_pipeline::{GpuMarchingCubesComputePipeline, MarchingCubesComputePipeline, MarchingCubesComputePipelineAsset};

use bevy::prelude::*;
use mc_render_core::MarchingCubesRenderPass;
use mc_render_pipeline::{GpuMarchingCubesRenderPipeline, MarchingCubesRenderPipeline, MarchingCubesRenderPipelineAsset};
use wde_wgpu::buffer::BufferUsage;

use wde_render::{assets::{Buffer, RenderAssetsPlugin}, core::{Extract, Render, RenderApp, RenderSet}, passes::render_graph::RenderGraph};

pub mod mc_compute_core;
pub mod mc_compute_pipeline;
pub mod mc_render_core;
pub mod mc_render_pipeline;

pub struct TerrainFeaturesPlugin;
impl Plugin for TerrainFeaturesPlugin {
    fn build(&self, app: &mut App) {
        // COMPUTE PASS
        // Manage chunks creation / deletion
        app
            .init_resource::<MarchingCubesHandler>()
            .add_systems(Update, MarchingCubesComputeTask::handle_tasks);

        // Manage chunks data extraction to the render thread
        app.get_sub_app_mut(RenderApp).unwrap()
            .init_resource::<MarchingCubesHandlerGPU>()
            .add_systems(Extract, MarchingCubesComputeTask::extract_chunks_data);

        // Manage chunks data generation on the render thread
        app
            .init_asset::<MarchingCubesComputePipelineAsset>()
            .add_plugins(RenderAssetsPlugin::<GpuMarchingCubesComputePipeline>::default());
        app.get_sub_app_mut(RenderApp).unwrap()
            .add_systems(Render, MarchingCubesComputeTask::generate_chunks_compute.in_set(RenderSet::PrepareAssets));

        // RENDER PASS
        // Pipelines
        app
            .init_asset::<MarchingCubesRenderPipelineAsset>()
            .add_plugins(RenderAssetsPlugin::<GpuMarchingCubesRenderPipeline>::default());

        // Render pass
        let mut render_graph = app.get_sub_app_mut(RenderApp).unwrap()
            .world_mut().get_resource_mut::<RenderGraph>().unwrap();
        render_graph.add_pass::<MarchingCubesRenderPass>(100);
    }

    fn finish(&self, app: &mut App) {
        // COMPUTE PASS
        // Create the compute pipeline
        let pipeline = app.world_mut()
            .get_resource::<AssetServer>().unwrap().add(MarchingCubesComputePipelineAsset);
        app.get_sub_app_mut(RenderApp).unwrap().world_mut().spawn(MarchingCubesComputePipeline(pipeline));

        // Create the staging buffer
        let staging_buffer = Buffer {
            label: "marching-cubes-desc-staging-cpu".to_string(),
            size: std::mem::size_of::<GpuMarchingCubesDescription>(),
            usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            content: None
        };
        let staging_buffer = app.world_mut().get_resource::<AssetServer>().unwrap().add(staging_buffer);
        app.world_mut().get_resource_mut::<MarchingCubesHandler>().unwrap().desc_buffer_cpu = Some(staging_buffer);


        // RENDER PASS
        // Create the render pipeline
        let pipeline = app.world_mut()
            .get_resource::<AssetServer>().unwrap().add(MarchingCubesRenderPipelineAsset);
        app.get_sub_app_mut(RenderApp).unwrap().world_mut().spawn(MarchingCubesRenderPipeline(pipeline));
    }
}