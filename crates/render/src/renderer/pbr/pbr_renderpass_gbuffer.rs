use std::collections::HashMap;

use bevy::prelude::*;
use crate::{assets::{materials::PbrMaterial, GpuBuffer, GpuMaterial, GpuMesh, GpuTexture, Mesh, RenderAssets}, components::TransformUniform, core::extract_macros::ExtractWorld, features::CameraFeatureRender, pipelines::{CachedPipelineStatus, PipelineManager}, renderer::depth::DepthTexture};
use wde_wgpu::{command_buffer::{RenderPassBuilder, RenderPassColorAttachment, RenderPassDepth, WCommandBuffer}, instance::WRenderInstance};

use super::{GpuPbrGBufferRenderPipeline, PbrDeferredTextures, PbrSsbo};

pub struct PbrGBufferRenderBatch {
    mesh: Handle<Mesh>,
    material: Handle<PbrMaterial>,
    first: usize,
    count: usize,
    index_count: usize,
}
#[derive(Resource)]
pub struct PbrGBufferRenderPass {
    /// The order of the batches: (mesh, material) -> [batch index].
    pub batches_order: HashMap<(AssetId<Mesh>, AssetId<PbrMaterial>), Vec<usize>>,
    /// The render batches.
    pub batches: Vec<PbrGBufferRenderBatch>,
}
impl PbrGBufferRenderPass {
    /// Create the batches with the correct mesh and material.
    pub fn create_batches(
        mut pass: ResMut<PbrGBufferRenderPass>, render_instance: Res<WRenderInstance<'static>>,
        entities: ExtractWorld<Query<(&Transform, &Handle<Mesh>, &Handle<PbrMaterial>)>>,
        meshes: Res<RenderAssets<GpuMesh>>, materials: Res<RenderAssets<GpuMaterial<PbrMaterial>>>,
        buffers: Res<RenderAssets<GpuBuffer>>, ssbo: Res<PbrSsbo>
    ) {
        // Clear the batches of the previous frame
        pass.batches.clear();
        pass.batches_order.clear();

        // Get the ssbo
        let ssbo_bf = match buffers.get(&ssbo.buffer) {
            Some(ssbo) => ssbo,
            None => return
        };
        
        // If no entities, return
        if entities.is_empty() {
            return
        }

        // Create the batches
        let render_instance = render_instance.data.read().unwrap();
        ssbo_bf.buffer.map_write(&render_instance, |mut view| {
            let mut first = 0;
            let mut count = 1;
            let mut last_mesh: Option<Handle<Mesh>> = None;
            let mut last_material: Option<Handle<PbrMaterial>> = None;
            let data = view.as_mut_ptr() as *mut TransformUniform;

            for (transform, mesh_handle, material_handle) in entities.iter() {
                // Check if new element in same batch
                let last_mesh_ref = last_mesh.as_ref();
                let last_material_ref = last_material.as_ref();
                if last_mesh_ref.is_some() && last_material_ref.is_some() {
                    if mesh_handle.id() == last_mesh_ref.unwrap().id() && material_handle.id() == last_material_ref.unwrap().id() {
                        // Update the ssbo
                        let transform = TransformUniform::new(transform);
                        unsafe {
                            *data.add(first + count) = transform;
                        }

                        // Increment the count
                        count += 1;

                        continue;
                    } else {
                        // Push the batch
                        pass.batches.push(PbrGBufferRenderBatch {
                            mesh: last_mesh_ref.unwrap().clone_weak(),
                            material: last_material_ref.unwrap().clone_weak(),
                            first,
                            count,
                            index_count: match meshes.get(last_mesh_ref.unwrap()) {
                                Some(mesh) => mesh.index_count as usize,
                                None => 0
                            }
                        });

                        let batch_index = pass.batches.len() - 1;
                        pass.batches_order.entry(
                            (last_mesh_ref.unwrap().id(), last_material_ref.unwrap().id())
                        ).or_default().push(batch_index);


                        // Reset the batch
                        first += count;
                        count = 1;
                        last_mesh = None;
                        last_material = None;
                    }
                }

                // Update the last mesh and ssbo if loaded
                let mut updated_mesh = false;
                let mut updated_material = false;
                if meshes.get(mesh_handle).is_some() {
                    last_mesh = Some(mesh_handle.clone_weak());
                    updated_mesh = true;
                }
                if materials.get(material_handle).is_some() {
                    last_material = Some(material_handle.clone_weak());
                    updated_material = true;
                }
                if updated_mesh && updated_material {
                    // Update the ssbo
                    let transform = TransformUniform::new(transform);
                    unsafe {
                        *data.add(first) = transform;
                    }
                }
            }

            // Push the last batch
            if let (Some(last_mesh), Some(last_material)) = (last_mesh, last_material) {
                pass.batches.push(PbrGBufferRenderBatch {
                    mesh: last_mesh.clone_weak(),
                    material: last_material.clone_weak(),
                    first,
                    count,
                    index_count: match meshes.get(&last_mesh) {
                        Some(mesh) => mesh.index_count as usize,
                        None => 0
                    }
                });

                let batch_index = pass.batches.len() - 1;
                pass.batches_order.entry(
                    (last_mesh.id(), last_material.id())
                ).or_default().push(batch_index);
            }
        });

        // Update the ssbo
        let ssbo_gpu = match buffers.get(&ssbo.buffer_gpu) {
            Some(ssbo) => ssbo,
            None => return
        };
        ssbo_gpu.buffer.copy_from_buffer(&render_instance, &ssbo_bf.buffer);
    }


    /// Render the different batches.
    pub fn render_g_buffer(
        (render_instance, pipeline_manager): (
            Res<WRenderInstance<'static>>,  Res<PipelineManager>
        ),
        (camera_layout, ssbo) : (Res<CameraFeatureRender>, Res<PbrSsbo>),
        (meshes, textures, materials): (
            Res<RenderAssets<GpuMesh>>, Res<RenderAssets<GpuTexture>>, Res<RenderAssets<GpuMaterial<PbrMaterial>>>
        ),
        (gbuffer_pipeline, render_mesh_pass, depth_texture): (
            Res<RenderAssets<GpuPbrGBufferRenderPipeline>>, Res<PbrGBufferRenderPass>, Res<DepthTexture>
        ),
        deferred_textures: Res<PbrDeferredTextures>
    ) {
        // Get the render instance and swapchain frame
        let render_instance = render_instance.data.read().unwrap();

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
        let gbuffer_pipeline = match gbuffer_pipeline.iter().next() {
            Some((_, pipeline)) => pipeline,
            None => return
        };

        // Check if the deferred textures are ready
        let (albedo, normal, material_tex) = match (
            textures.get(&deferred_textures.albedo),
            textures.get(&deferred_textures.normal), textures.get(&deferred_textures.material)
        ) {
            (Some(albedo), Some(normal), Some(material_tex))
                => (albedo, normal, material_tex),
            _ => return
        };

        // Create the render pass
        let mut command_buffer = WCommandBuffer::new(&render_instance, "gbuffer-pbr");
        {
            let mut render_pass = command_buffer.create_render_pass("gbuffer-pbr", |builder: &mut RenderPassBuilder| {
                builder.set_depth_texture(RenderPassDepth {
                    texture: Some(&depth_texture.texture.view),
                    ..Default::default()
                });
                builder.add_color_attachment(RenderPassColorAttachment {
                    texture: Some(&albedo.texture.view),
                    ..Default::default()
                });
                builder.add_color_attachment(RenderPassColorAttachment {
                    texture: Some(&normal.texture.view),
                    ..Default::default()
                });
                builder.add_color_attachment(RenderPassColorAttachment {
                    texture: Some(&material_tex.texture.view),
                    ..Default::default()
                });
            });

            // Render the mesh
            if let (
                CachedPipelineStatus::OkRender(pipeline),
                Some(camera_bg),
                Some(ssbo_bind_group)
            ) = (
                pipeline_manager.get_pipeline(gbuffer_pipeline.cached_pipeline_index),
                &camera_layout.bind_group,
                &ssbo.bind_group
            ) {
                // Set the camera bind group
                render_pass.set_bind_group(0, camera_bg);

                // Set the pipeline
                if render_pass.set_pipeline(pipeline).is_ok() {
                    // Set the ssbo
                    render_pass.set_bind_group(1, ssbo_bind_group);

                    let mut old_mesh_id = None;
                    let mut old_material_id = None;

                    // For each set of mesh and material
                    for (_, batch_index) in render_mesh_pass.batches_order.iter() {
                        // For each batch of the set
                        for &batch_index in batch_index.iter() {
                            let batch = render_mesh_pass.batches.get(batch_index).unwrap();
                        
                            // Set the material
                            if old_material_id != Some(batch.material.id()) {
                                let material = match materials.get(&batch.material) {
                                    Some(material) => material,
                                    None => continue // Should not happen
                                };

                                // Set the material bind group
                                render_pass.set_bind_group(2, &material.bind_group);
                                old_material_id = Some(batch.material.id());
                            }

                            // Set the mesh
                            if old_mesh_id != Some(batch.mesh.id()) {
                                let mesh = match meshes.get(&batch.mesh) {
                                    Some(mesh) => mesh,
                                    None => continue // Should not happen
                                };

                                // Set the mesh buffers
                                render_pass.set_vertex_buffer(0, &mesh.vertex_buffer);
                                render_pass.set_index_buffer(&mesh.index_buffer);
                                old_mesh_id = Some(batch.mesh.id());
                            }

                            // Draw the mesh
                            let instance_indices = batch.first as u32..((batch.first + batch.count) as u32);
                            match render_pass.draw_indexed(0..batch.index_count as u32, instance_indices) {
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
