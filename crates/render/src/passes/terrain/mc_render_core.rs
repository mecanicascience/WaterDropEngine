use bevy::prelude::*;
use crate::{assets::{GpuBuffer, GpuTexture, RenderAssets, RenderAssetsPlugin}, core::{RenderApp, SwapchainFrame}, features::CameraFeatureRender, pipelines::{CachedPipelineStatus, PipelineManager}, passes::depth::DepthTexture};
use wde_wgpu::{command_buffer::{RenderPassBuilder, RenderPassColorAttachment, RenderPassDepth, WCommandBuffer, WLoadOp}, instance::WRenderInstance};

use super::{mc_compute_core::MarchingCubesHandlerGPU, mc_render_pipeline::{GpuMarchingCubesRenderPipeline, MarchingCubesRenderPipeline, MarchingCubesRenderPipelineAsset}};

pub struct MarchingCubesRenderPass;
impl Plugin for MarchingCubesRenderPass {
    fn build(&self, app: &mut App) {
        // Pipelines
        app
            .init_asset::<MarchingCubesRenderPipelineAsset>()
            .add_plugins(RenderAssetsPlugin::<GpuMarchingCubesRenderPipeline>::default());
    }

    fn finish(&self, app: &mut App) {
        // Create the render pipeline
        let pipeline = app.world_mut()
            .get_resource::<AssetServer>().unwrap().add(MarchingCubesRenderPipelineAsset);
        app.get_sub_app_mut(RenderApp).unwrap().world_mut().spawn(MarchingCubesRenderPipeline(pipeline));
    }
}

impl MarchingCubesRenderPass {
    /// Render the different batches.
    pub fn render_terrain(
        (render_instance, swapchain_frame, pipeline_manager): (
            Res<WRenderInstance<'static>>, Res<SwapchainFrame>,  Res<PipelineManager>
        ),
        camera_layout : Res<CameraFeatureRender>,
        (textures, buffers): (
            Res<RenderAssets<GpuTexture>>, Res<RenderAssets<GpuBuffer>>
        ),
        (mc_pipeline, depth_texture): (
            Res<RenderAssets<GpuMarchingCubesRenderPipeline>>, Res<DepthTexture>
        ),
        handler: Res<MarchingCubesHandlerGPU>,
    ) {
        // Get the render instance and swapchain frame
        let render_instance = render_instance.data.read().unwrap();
        let swapchain_frame = swapchain_frame.data.as_ref().unwrap();

        // Check if depth texture is ready
        let depth_texture = match textures.get(&depth_texture.texture) {
            Some(tex) => if render_instance.surface_config.as_ref().unwrap().width == tex.texture.size.0
                && render_instance.surface_config.as_ref().unwrap().height == tex.texture.size.1 {
                tex
            } else {
                return
            },
            None => return
        };

        // Check if pipeline is ready
        let mcbuffer_pipeline = match mc_pipeline.iter().next() {
            Some((_, pipeline)) => pipeline,
            None => return
        };

        // Test if swapchain frame and depth texture have the same size
        if swapchain_frame.texture.texture.size().width != depth_texture.texture.size.0 || swapchain_frame.texture.texture.size().height != depth_texture.texture.size.1 {
            warn!("Swapchain frame and depth texture have different sizes: {:?} vs {:?}.", swapchain_frame.texture.texture.size(), depth_texture.texture.size);
            return;
        }
        
        // Create the render pass
        let mut command_buffer = WCommandBuffer::new(&render_instance, "marching-cubes");
        {
            let mut render_pass = command_buffer.create_render_pass("marching-cubes", |builder: &mut RenderPassBuilder| {
                builder.add_color_attachment(RenderPassColorAttachment {
                    texture: Some(&swapchain_frame.view),
                    load: WLoadOp::Load,
                    ..Default::default()
                });
                builder.set_depth_texture(RenderPassDepth {
                    texture: Some(&depth_texture.texture.view),
                    load_operation: WLoadOp::Load,
                    ..Default::default()
                });
            });

            // Render the mesh
            if let (
                CachedPipelineStatus::OkRender(pipeline),
                Some(camera_bg)
            ) = (
                pipeline_manager.get_pipeline(mcbuffer_pipeline.cached_pipeline_index),
                &camera_layout.bind_group
            ) {
                // Set the camera bind group
                render_pass.set_bind_group(0, camera_bg);

                // Set the pipeline
                if render_pass.set_pipeline(pipeline).is_ok() {
                    for (_, chunk) in handler.active_chunks.iter() {
                        // Check if the chunk is generated
                        if !chunk.generated {
                            continue;
                        }

                        // Get the vertex and index buffers
                        if let (
                            Some(vertex_buffer),
                            Some(index_buffer)
                        ) = (
                            buffers.get(&chunk.vertices),
                            buffers.get(&chunk.indices)
                        ) {
                            // Set the mesh buffers
                            render_pass.set_vertex_buffer(0, &vertex_buffer.buffer);
                            render_pass.set_index_buffer(&index_buffer.buffer);

                            // Draw the mesh
                            match render_pass.draw_indexed(0..chunk.indices_counter, 0..1) {
                                Ok(_) => {},
                                Err(e) => {
                                    error!("Failed to draw: {:?}.", e);
                                }
                            };
                        }
                    }
                } else {
                    error!("Failed to set pipeline.");
                }
            }
        }

        // Submit the command buffer
        command_buffer.submit(&render_instance);
    }
}
