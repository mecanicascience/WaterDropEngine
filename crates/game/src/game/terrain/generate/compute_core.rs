use bevy::prelude::*;
use wde_render::{assets::{GpuBuffer, RenderAssets}, pipelines::{CachedPipelineStatus, PipelineManager}};
use wde_wgpu::{bind_group::BindGroup, command_buffer::WCommandBuffer, instance::WRenderInstance};

use crate::terrain::{mc_chunk::{MCChunksList, MCLoadingChunk, MCPendingChunk}, mc_compute_main::{GpuMarchingCubesDescription, MCComputeHandlerGPU}};

use super::compute_pipeline::GpuMCComputePipelineGenerate;

pub struct MCComputeCorePoints;
impl MCComputeCorePoints {
    /** Create the bind groups if they are not already created. */
    pub fn create_bind_groups(
        handler: Res<MCComputeHandlerGPU>, buffers: Res<RenderAssets<GpuBuffer>>,
        render_instance: Res<WRenderInstance<'static>>, mut pipeline: ResMut<RenderAssets<GpuMCComputePipelineGenerate>>,
        mut loading_chunks: Query<&mut MCLoadingChunk>
    ) {
        // Get the compute pipeline
        let compute_pipeline = match pipeline.iter_mut().next() {
            Some((_, pipeline)) => pipeline,
            None => return
        };

        // Create the bind groups for the handler if they are not already created
        if compute_pipeline.desc_gpu_group.is_none() && handler.desc_gpu.is_some() {
            // Get the layouts
            let (desc_gpu_layout, vertices_layout) = match (
                &compute_pipeline.desc_gpu_layout,
                &compute_pipeline.vertices_gpu_layout
            ) {
                (Some(desc_gpu_layout), Some(vertices_layout)) => (
                    desc_gpu_layout, vertices_layout
                ),
                _ => return
            };

            // Get the buffers
            let (desc_gpu, vertices) = match (
                buffers.get(handler.desc_gpu.as_ref().unwrap()),
                buffers.get(handler.vertices_gpu.as_ref().unwrap()),
            ) {
                (Some(desc_gpu), Some(vertices)) => (
                    desc_gpu, vertices
                ),
                _ => return
            };

            // Create the bind groups
            let render_instance = render_instance.data.read().unwrap();
            let desc_gpu_bind_group = BindGroup::build(
                "marching-cubes-generate-desc-gpu", &render_instance, &desc_gpu_layout.build(&render_instance),
                &vec![BindGroup::buffer(0, &desc_gpu.buffer)]);
            let vertices_bind_group = BindGroup::build(
                "marching-cubes-generate-vertices-gpu", &render_instance, &vertices_layout.build(&render_instance),
                &vec![BindGroup::buffer(0, &vertices.buffer)]);

            // Update the handler
            compute_pipeline.desc_gpu_group = Some(desc_gpu_bind_group);
            compute_pipeline.vertices_gpu_group = Some(vertices_bind_group);
        }

        // Create the bind groups for the chunks if they are not already created
        let render_instance = render_instance.data.read().unwrap();
        for mut chunk in loading_chunks.iter_mut() {
            if chunk.points_gpu_group.is_none() {
                // Get the layout
                let points_layout = match &compute_pipeline.points_gpu_layout {
                    Some(points_layout) => points_layout,
                    _ => continue
                };

                // Get the buffer
                let points = match buffers.get(&chunk.points_gpu) {
                    Some(points) => points,
                    _ => continue
                };

                // Create the bind group
                let points_bind_group = BindGroup::build(
                    "marching-cubes-points-gpu", &render_instance, &points_layout.build(&render_instance),
                    &vec![BindGroup::buffer(0, &points.buffer)]);

                // Update the chunk
                chunk.points_gpu_group = Some(points_bind_group);
            }
        }
    }
    
    /** Run the compute pass to generate the chunks. */
    pub fn compute(
        handler: Res<MCComputeHandlerGPU>, mut buffers: ResMut<RenderAssets<GpuBuffer>>,
        render_instance: Res<WRenderInstance<'static>>, pipeline: Res<RenderAssets<GpuMCComputePipelineGenerate>>,
        pipeline_manager: ResMut<PipelineManager>, query: Query<(Entity, &MCLoadingChunk)>,
        chunks_list: ResMut<MCChunksList>, mut commands: Commands
    ) {
        // Get the compute pipeline
        let compute_pipeline = match pipeline.iter().next() {
            Some((_, pipeline)) => pipeline,
            None => return
        };

        // Check if the handler is ready
        let (desc_buffer_cpu, desc_buffer_gpu, desc_buffer_group, vertices_cpu, vertices_gpu, vertices_group) = match (
            &handler.desc_cpu, &handler.desc_gpu, &compute_pipeline.desc_gpu_group, &handler.vertices_cpu, &handler.vertices_gpu, &compute_pipeline.vertices_gpu_group
        ) {
            (Some(desc_buffer_cpu), Some(desc_buffer_gpu), Some(desc_buffer_group), Some(vertices_cpu), Some(vertices_gpu), Some(vertices_group)) => (
                desc_buffer_cpu, desc_buffer_gpu, desc_buffer_group, vertices_cpu, vertices_gpu, vertices_group
            ),
            _ => return
        };

        // Generate the chunks
        for (entity, chunk) in query.iter() {
            let desc = chunks_list.chunks.get(&chunk.index).unwrap();

            // Update the description buffer
            trace!("Running the compute shader for the chunk {:?}.", chunk.index);
            let desc_buff = GpuMarchingCubesDescription {
                translation: [desc.translation.x, desc.translation.y, desc.translation.z, 0.0],
                chunk_length: [desc.chunk_length.x, desc.chunk_length.y, desc.chunk_length.z, 0.0],
                chunk_sub_count: [desc.chunk_sub_count.x, desc.chunk_sub_count.y, desc.chunk_sub_count.z, 0],
                triangles_counter: 0,
                iso_level: desc.iso_level,
                padding: [0, 0]
            };
            let render_instance = render_instance.data.read().unwrap();
            buffers.get_mut(desc_buffer_gpu).unwrap().buffer.write(&render_instance, bytemuck::cast_slice(&[desc_buff]), 0);

            // Create the compute pass
            let mut generated = false;
            let mut command_buffer = WCommandBuffer::new(&render_instance, "marching-cubes-generate");
            {
                let mut compute_pass = command_buffer.create_compute_pass("marching-cubes-generate");

                // Set the pipeline
                if let (
                    CachedPipelineStatus::OkCompute(pipeline),
                    Some(points_group)
                ) = (
                    pipeline_manager.get_pipeline(compute_pipeline.cached_pipeline_index),
                    &chunk.points_gpu_group
                ) {
                    if compute_pass.set_pipeline(pipeline).is_err() {
                        continue;
                    }

                    // Set the bind groups
                    compute_pass.set_bind_group(0, desc_buffer_group);
                    compute_pass.set_bind_group(1, points_group);
                    compute_pass.set_bind_group(2, vertices_group);

                    // Dispatch the compute pass
                    const NUM_THREADS: i32 = 10;
                    let dispatch_count_x = (desc.chunk_sub_count.x as f32 / NUM_THREADS as f32).ceil() as u32;
                    let dispatch_count_y = (desc.chunk_sub_count.y as f32 / NUM_THREADS as f32).ceil() as u32;
                    let dispatch_count_z = (desc.chunk_sub_count.z as f32 / NUM_THREADS as f32).ceil() as u32;
                    trace!("Dispatching the compute pass for generating chunk {:?} with marching cubes with {} threads and {:?} dispatches.", entity, NUM_THREADS, [dispatch_count_x, dispatch_count_y, dispatch_count_z]);
                    if let Err(e) = compute_pass.dispatch(dispatch_count_x, dispatch_count_y, dispatch_count_z) {
                        error!("Failed to dispatch the compute pass for generating chunk {:?} with marching cubes: {:?}", entity, e);
                        continue;
                    }
                    generated = true;
                }
            }
            if !generated {
                continue;
            }

            // Submit the command buffer
            command_buffer.submit(&render_instance);

            // Read the indices counter
            let mut triangles_count = 0;
            let mut c_sub_count = [0, 0, 0];
            let cpu_buff = &buffers.get(desc_buffer_cpu).unwrap().buffer;
            cpu_buff.copy_from_buffer(&render_instance, &buffers.get(desc_buffer_gpu).unwrap().buffer);
            cpu_buff.map_read(&render_instance, |data| {
                let desc = bytemuck::from_bytes::<GpuMarchingCubesDescription>(&data);
                triangles_count = desc.triangles_counter;
                c_sub_count = [desc.chunk_sub_count[0] as usize, desc.chunk_sub_count[1] as usize, desc.chunk_sub_count[2] as usize];
            });

            // Warn if the indices counter is too high
            let vertex_buffer_size = std::mem::size_of::<[f32; 12]>() * 5 * c_sub_count[0] * c_sub_count[1] * c_sub_count[2];
            if std::mem::size_of::<[f32; 12]>() * (triangles_count as usize) > vertex_buffer_size {
                error!("In the marching cubes algorithm, there is too much vertices overflowing the vertices buffer. The buffer size is {} and the indices counter is {}.", vertex_buffer_size, triangles_count);
                continue;
            }

            // Read and process the vertices
            let mut raw_vertices: Vec<f32> = Vec::new();
            let vertices_buff = &buffers.get(vertices_cpu).unwrap().buffer;
            vertices_buff.copy_from_buffer(&render_instance, &buffers.get(vertices_gpu).unwrap().buffer);
            vertices_buff.map_read(&render_instance, |data| {
                let vertices: &[f32] = bytemuck::cast_slice(&data);
                raw_vertices.extend_from_slice(&vertices[..triangles_count as usize * 12]);
            });

            // Spawn the chunk entity
            debug!("Generated {} raw triangles for chunk {:?}.", triangles_count, chunk.index);
            commands.entity(entity).despawn();
            commands.spawn(MCPendingChunk {
                index: chunk.index,
                raw_triangles: raw_vertices,
                triangles_counter: triangles_count,
                points_gpu: chunk.points_gpu.clone()
            });
        }
    }
}
