use bevy::prelude::*;
use crate::{assets::{GpuMesh, GpuTexture, Mesh, ModelBoundingBox, RenderAssets}, core::{extract_macros::ExtractWorld, SwapchainFrame}, pipelines::{CachedPipelineStatus, PipelineManager}, renderer::depth::DepthTexture};
use wde_wgpu::{command_buffer::{RenderPassBuilder, RenderPassColorAttachment, RenderPassDepth, WCommandBuffer, WLoadOp, WStoreOp}, instance::WRenderInstance, vertex::WVertex};

use super::{GpuPbrLightingRenderPipeline, PbrDeferredTexturesLayout};

#[derive(Resource, Default)]
pub struct PbrLightingRenderPassMesh {
    pub deferred_mesh: Option<Handle<Mesh>>
}
impl PbrLightingRenderPassMesh {
    // Creates the rendering mesh.
    pub fn init(assets_server: Res<AssetServer>, mut render_pass: ResMut<PbrLightingRenderPassMesh>) {
        // Create the 2d quad mesh
        let deferred_mesh: Handle<Mesh> = assets_server.add(Mesh {
            label: "deferred-lighting-pass".to_string(),
            vertices: vec![
                WVertex { position: [-1.0, 1.0, 0.0], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [-1.0, -1.0, 0.0], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [1.0, -1.0, 0.0], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [1.0, 1.0, 0.0], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            bounding_box: ModelBoundingBox {
                min: Vec3::new(-1.0, -1.0, 0.0),
                max: Vec3::new(1.0, 1.0, 0.0),
            },
        });
        render_pass.deferred_mesh = Some(deferred_mesh);
    }

    /// Extract the texture handle every frame
    pub fn extract_mesh(mesh_cpu: ExtractWorld<Res<PbrLightingRenderPassMesh>>, mut render_pass: ResMut<PbrLightingRenderPassMesh>) {
        render_pass.deferred_mesh = None;
        if let Some(ref mesh_cpu) = mesh_cpu.deferred_mesh {
            render_pass.deferred_mesh = Some(mesh_cpu.clone_weak());
        }
    }
}

#[derive(Resource)]
pub struct PbrLightingRenderPass;
impl PbrLightingRenderPass {
    /// Render the different batches.
    pub fn render_lighting(
        (render_instance, swapchain_frame, pipeline_manager): (
            Res<WRenderInstance<'static>>, Res<SwapchainFrame>,  Res<PipelineManager>
        ),
        (textures, meshes): (
            Res<RenderAssets<GpuTexture>>, Res<RenderAssets<GpuMesh>>
        ),
        (lighting_pipeline, deferred_mesh, textures_layout): (
            Res<RenderAssets<GpuPbrLightingRenderPipeline>>, Res<PbrLightingRenderPassMesh>, Res<PbrDeferredTexturesLayout>
        ),
        depth_texture: Res<DepthTexture>
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

        // Check if mesh is ready
        let deferred_mesh = match &deferred_mesh.deferred_mesh {
            Some(mesh) => match meshes.get(mesh) {
                Some(mesh) => mesh,
                None => return
            },
            None => return
        };

        // Check if pipeline is ready
        let lighting_pipeline = match lighting_pipeline.iter().next() {
            Some((_, pipeline)) => pipeline,
            None => return
        };

        // Create the render pass
        let mut command_buffer = WCommandBuffer::new(&render_instance, "lighting-pbr");
        {
            let mut render_pass = command_buffer.create_render_pass("lighting-pbr", |builder: &mut RenderPassBuilder| {
                builder.set_depth_texture(RenderPassDepth {
                    texture: Some(&depth_texture.texture.view),
                    load_operation: WLoadOp::Load,
                    store_operation: WStoreOp::Discard
                });
                builder.add_color_attachment(RenderPassColorAttachment {
                    texture: Some(&swapchain_frame.view),
                    ..Default::default()
                });
            });

            // Render the mesh
            if let (
                CachedPipelineStatus::Ok(pipeline),
                Some(deferred_bind_group)
            ) = (
                pipeline_manager.get_pipeline(lighting_pipeline.cached_pipeline_index),
                &textures_layout.deferred_bind_group
            ) {
                // Set the pipeline
                if render_pass.set_pipeline(pipeline).is_ok() {
                    // Get the mesh
                    render_pass.set_vertex_buffer(0, &deferred_mesh.vertex_buffer);
                    render_pass.set_index_buffer(&deferred_mesh.index_buffer);

                    // Set bind group
                    render_pass.set_bind_group(0, deferred_bind_group);

                    // Draw the mesh
                    match render_pass.draw_indexed(0..deferred_mesh.index_count, 0..1) {
                        Ok(_) => {},
                        Err(e) => {
                            error!("Failed to draw: {:?}.", e);
                        }
                    };
                } else {
                    error!("Failed to set pipeline.");
                }
            }
        }

        // Submit the command buffer
        command_buffer.submit(&render_instance);
    }
}
