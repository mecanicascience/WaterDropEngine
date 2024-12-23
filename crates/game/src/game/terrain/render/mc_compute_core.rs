use bevy::{ecs::world::CommandQueue, prelude::*, tasks::{block_on, futures_lite::future, Task}, utils::HashMap};
use wde_render::{assets::{Buffer, GpuBuffer, RenderAssets}, core::extract_macros::ExtractWorld, pipelines::{CachedPipelineStatus, PipelineManager}};
use wde_wgpu::{bind_group::{BindGroup, WgpuBindGroup}, buffer::BufferUsage, command_buffer::WCommandBuffer, instance::WRenderInstance, vertex::WVertex};

use super::mc_compute_pipeline::GpuMarchingCubesComputePipeline;

pub type ChunkIndex = (i32, i32, i32);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
pub struct GpuMarchingCubesDescription {
    pub index: [f32; 4], // (x, y, z, 0)
    pub translation: [f32; 4], // (x, y, z, 0)
    pub chunk_length: [f32; 4], // (x, y, z, 0)
    pub chunk_sub_count: [u32; 4], // (x, y, z, 0)
    pub indices_counter: u32,
    pub iso_level: f32,
    pub padding: [f32; 2]
}

#[derive(Clone)]
pub struct MarchingCubesChunkDescription {
    pub index: ChunkIndex,
    pub translation: Vec3,
    pub chunk_length: Vec3,
    pub chunk_sub_count: UVec3,
    pub iso_level: f32,
    pub f: fn(Vec3) -> f32
}

pub struct MarchingCubesChunk {
    // Description of the chunk
    pub description: MarchingCubesChunkDescription,
    pub generated: bool,
    pub indices_counter: u32,

    // Buffers
    pub desc_gpu: Handle<Buffer>,
    pub points: Handle<Buffer>,
    pub vertices: Handle<Buffer>,
    pub indices: Handle<Buffer>,

    // Bind groups
    pub desc_gpu_group: Option<WgpuBindGroup>,
    pub points_group: Option<WgpuBindGroup>,
    pub triangles_group: Option<WgpuBindGroup>
}
impl Clone for MarchingCubesChunk {
    fn clone(&self) -> Self {
        MarchingCubesChunk {
            description: self.description.clone(),
            generated: self.generated,
            indices_counter: self.indices_counter,

            desc_gpu: self.desc_gpu.clone(),
            points: self.points.clone(),
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),

            desc_gpu_group: None,
            points_group: None,
            triangles_group: None
        }
    }
}

#[derive(Resource, Default)]
pub struct MarchingCubesHandler {
    pub active_chunks: Vec<MarchingCubesChunk>,
    pub desc_buffer_cpu: Option<Handle<Buffer>>,
}
#[derive(Resource, Default)]
pub struct MarchingCubesHandlerGPU {
    pub active_chunks: HashMap<ChunkIndex, MarchingCubesChunk>,
    pub loading_chunks: HashMap<ChunkIndex, MarchingCubesChunk>,

    // Buffers
    pub desc_buffer_cpu: Option<Handle<Buffer>>,
}


#[derive(Component)]
pub struct MarchingCubesComputeTask(pub Task<CommandQueue>);
impl MarchingCubesComputeTask {
    /**
     * Generate a new chunk with the given description (chunk id).
     * This will generate the points, vertices and indices buffers, and add the chunk to the loading chunks.
     */
    pub fn generate_new_chunk(task_entity: Entity, desc: MarchingCubesChunkDescription, max_buffer_size: usize) -> CommandQueue {
        // Generate the description buffer for the GPU
        let desc_buffer_gpu = Buffer {
            label: format!("marching-cubes-desc-gpu-{:?}", desc.index),
            size: std::mem::size_of::<GpuMarchingCubesDescription>(),
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
            content: None
        };

        // Generate the grid points
        let c_sub_count = desc.chunk_sub_count;
        let mut points = Vec::with_capacity((c_sub_count[0] * c_sub_count[1] * c_sub_count[2]) as usize);
        for i in 0..c_sub_count[0] {
            for j in 0..c_sub_count[1] {
                for k in 0..c_sub_count[2] {
                    let x = desc.translation.x - desc.chunk_length[0] / 2.0 + i as f32 * desc.chunk_length[0] / (c_sub_count[0] as f32 - 1.0);
                    let y = desc.translation.y - desc.chunk_length[1] / 2.0 + j as f32 * desc.chunk_length[1] / (c_sub_count[1] as f32 - 1.0);
                    let z = desc.translation.z - desc.chunk_length[2] / 2.0 + k as f32 * desc.chunk_length[2] / (c_sub_count[2] as f32 - 1.0);
                    points.push([x, y, z, (desc.f)(Vec3::new(x, y, z))]);
                }
            }
        }
        let points_buffer = Buffer {
            label: format!("marching-cubes-points-{:?}", desc.index),
            size: std::cmp::min(std::mem::size_of::<[f32; 4]>() * (c_sub_count[0] * c_sub_count[1] * c_sub_count[2]) as usize, max_buffer_size),
            usage: BufferUsage::STORAGE,
            content: Some(bytemuck::cast_slice(&points).to_vec())
        };

        // Generate the vertices and indices buffers
        let vertex_buffer = Buffer {
            label: format!("marching-cubes-vertices-{:?}", desc.index),
            size: std::cmp::min(std::mem::size_of::<WVertex>() * 3 * 5 * (c_sub_count[0] * c_sub_count[1] * c_sub_count[2]) as usize, max_buffer_size),
            usage: BufferUsage::VERTEX | BufferUsage::STORAGE,
            content: None
        };
        let index_buffer = Buffer {
            label: format!("marching-cubes-indices-{:?}", desc.index),
            size: std::cmp::min(std::mem::size_of::<u32>() * 3 * 5 * (c_sub_count[0] * c_sub_count[1] * c_sub_count[2]) as usize, max_buffer_size),
            usage: BufferUsage::INDEX | BufferUsage::STORAGE,
            content: None
        };
        
        // Register the buffers and add the chunk to the active chunks
        let mut command_queue = CommandQueue::default();
        command_queue.push(move |world: &mut World| {
            debug!("Registering chunk {:?} with marching cubes in main thread.", desc.index);
            
            // Create the chunk
            let mut buffers = world.get_resource_mut::<Assets<Buffer>>().unwrap();
            let chunk = MarchingCubesChunk {
                description: desc,
                generated: false,
                indices_counter: 0,

                desc_gpu: buffers.add(desc_buffer_gpu),
                points: buffers.add(points_buffer),
                vertices: buffers.add(vertex_buffer),
                indices: buffers.add(index_buffer),

                desc_gpu_group: None,
                points_group: None,
                triangles_group: None
            };

            // Add the chunk to the active chunks
            let mut handler = world.get_resource_mut::<MarchingCubesHandler>().unwrap();
            handler.active_chunks.push(chunk);

            // Despawn the entity
            world.commands().entity(task_entity).despawn();
        });
        command_queue
    }

    /**
     * Check if the tasks are done.
     * If so, run the command queue that will add the buffers and the chunk to the loading chunks.
     * If the task is done, despawn the entity.
     */
    pub fn handle_tasks(mut commands: Commands, mut tasks: Query<&mut MarchingCubesComputeTask>) {
        for mut task in &mut tasks {
            if let Some(mut commands_queue) = block_on(future::poll_once(&mut task.0)) {
                // Add the commands to the main thread for the next frame
                commands.append(&mut commands_queue);
            }
        }
    }

    /**
     * Extract the new chunks in the main thread and add them to the loading chunks in the render thread.
     */
    pub fn extract_chunks_data(
        handler_update: ExtractWorld<Res<MarchingCubesHandler>>,
        mut handler_render: ResMut<MarchingCubesHandlerGPU>,
    ) {
        if handler_render.desc_buffer_cpu.is_none() && handler_update.desc_buffer_cpu.is_some() {
            handler_render.desc_buffer_cpu = Some(handler_update.desc_buffer_cpu.clone().unwrap());
        }
        for chunk in handler_update.active_chunks.iter() {
            if !handler_render.active_chunks.contains_key(&chunk.description.index) && !handler_render.loading_chunks.contains_key(&chunk.description.index) {
                handler_render.loading_chunks.insert(chunk.description.index, chunk.clone());
            }
        }
    }

    /**
     * Run the compute pass to generate the chunks.
     */
    pub fn generate_chunks_compute(
        mut handler: ResMut<MarchingCubesHandlerGPU>, mut buffers: ResMut<RenderAssets<GpuBuffer>>,
        render_instance: Res<WRenderInstance<'static>>, pipeline: Res<RenderAssets<GpuMarchingCubesComputePipeline>>,
        pipeline_manager: ResMut<PipelineManager>
    ) {
    // Check if the staging buffer is created
    let desc_buffer_cpu = match &handler.desc_buffer_cpu {
        Some(buffer_handler) => match buffers.get(buffer_handler) {
            Some(_) => buffer_handler.clone_weak(),
            None => return
        },
        None => return
    };

    // Generate the chunks
    let mut new_chunks = Vec::new();
    for (index, chunk) in handler.loading_chunks.iter_mut() {
        // Get the compute pipeline
        let compute_pipeline = match pipeline.iter().next() {
            Some((_, pipeline)) => pipeline,
            None => continue
        };

        // Create the bind groups for the buffers chunk if they are not already created
        if chunk.points_group.is_none() || chunk.triangles_group.is_none() {
            // Check if the layouts are already created
            let (desc_gpu_layout, points_layout, vertices_layout) = match (
                &compute_pipeline.desc_gpu_layout,
                &compute_pipeline.points_layout,
                &compute_pipeline.vertices_layout
            ) {
                (Some(desc_gpu_layout), Some(points_layout), Some(vertices_layout)) => (
                    desc_gpu_layout, points_layout, vertices_layout
                ),
                _ => continue
            };

            // Check if the buffers are already created
            let (desc_gpu, points, vertices, indices) = match (
                buffers.get(&chunk.desc_gpu),
                buffers.get(&chunk.points),
                buffers.get(&chunk.vertices),
                buffers.get(&chunk.indices)
            ) {
                (Some(desc), Some(points), Some(vertices), Some(indices)) => (
                    desc, points, vertices, indices
                ),
                _ => continue
            };

            // Create the bind groups
            let render_instance = render_instance.data.read().unwrap();
            let desc_gpu_bind_group = BindGroup::build(
                "desc-marching-cubes", &render_instance, &desc_gpu_layout.build(&render_instance),
                &vec![BindGroup::buffer(0, &desc_gpu.buffer)]);
            let points_bind_group = BindGroup::build(
                "points-marching-cubes", &render_instance, &points_layout.build(&render_instance),
                &vec![
                    BindGroup::buffer(0, &points.buffer)
                ]);
            let triangles_bind_group = BindGroup::build(
                "triangles-marching-cubes", &render_instance, &vertices_layout.build(&render_instance),
                &vec![
                    BindGroup::buffer(0, &vertices.buffer),
                    BindGroup::buffer(1, &indices.buffer)
                ]);

            // Update the chunk
            chunk.desc_gpu_group = Some(desc_gpu_bind_group);
            chunk.points_group = Some(points_bind_group);
            chunk.triangles_group = Some(triangles_bind_group);
        }

        // Update the description buffer
        trace!("Generating chunk {:?} with marching cubes.", index);
        let desc_buff = GpuMarchingCubesDescription {
            index: [chunk.description.index.0 as f32, chunk.description.index.1 as f32, chunk.description.index.2 as f32, 0.0],
            translation: [chunk.description.translation.x, chunk.description.translation.y, chunk.description.translation.z, 0.0],
            chunk_length: [chunk.description.chunk_length.x, chunk.description.chunk_length.y, chunk.description.chunk_length.z, 0.0],
            chunk_sub_count: [chunk.description.chunk_sub_count.x, chunk.description.chunk_sub_count.y, chunk.description.chunk_sub_count.z, 0],
            indices_counter: 0,
            iso_level: chunk.description.iso_level,
            padding: [0.0; 2]
        };
        let render_instance = render_instance.data.read().unwrap();
        buffers.get_mut(&chunk.desc_gpu).unwrap().buffer.write(&render_instance, bytemuck::cast_slice(&[desc_buff]), 0);

        // Create the compute pass
        let mut command_buffer = WCommandBuffer::new(&render_instance, "marching-cubes");
        {
            let mut compute_pass = command_buffer.create_compute_pass("marching-cubes");

            // Set the pipeline
            if let (
                CachedPipelineStatus::OkCompute(pipeline),
                Some(desc_gpu_group),
                Some(points_group),
                Some(vertices_group)
            ) = (
                pipeline_manager.get_pipeline(compute_pipeline.cached_pipeline_index),
                &chunk.desc_gpu_group,
                &chunk.points_group,
                &chunk.triangles_group
            ) {
                if compute_pass.set_pipeline(pipeline).is_err() {
                    continue;
                }

                // Set the bind groups
                compute_pass.set_bind_group(0, desc_gpu_group);
                compute_pass.set_bind_group(1, points_group);
                compute_pass.set_bind_group(2, vertices_group);

                // Dispatch the compute pass
                let num_threads = 10;
                let dispatch_count_x = (chunk.description.chunk_sub_count.x as f32 / num_threads as f32).ceil() as u32;
                let dispatch_count_y = (chunk.description.chunk_sub_count.y as f32 / num_threads as f32).ceil() as u32;
                let dispatch_count_z = (chunk.description.chunk_sub_count.z as f32 / num_threads as f32).ceil() as u32;
                debug!("Dispatching the compute pass for generating chunk {:?} with marching cubes with {} threads and {:?} dispatches.", index, num_threads, [dispatch_count_x, dispatch_count_y, dispatch_count_z]);
                if let Err(e) = compute_pass.dispatch(dispatch_count_x, dispatch_count_y, dispatch_count_z) {
                    error!("Failed to dispatch the compute pass for generating chunk {:?} with marching cubes: {:?}", index, e);
                    continue;
                }
            }
            else {
                continue;
            }
        }

        // Submit the command buffer
        command_buffer.submit(&render_instance);

        // Read the indices counter
        let mut indices_counter = 0;
        let mut c_sub_count = [0, 0, 0];
        buffers.get(&desc_buffer_cpu).unwrap().buffer.copy_from_buffer(&render_instance, &buffers.get(&chunk.desc_gpu).unwrap().buffer);
        buffers.get(&desc_buffer_cpu).unwrap().buffer.map_read(&render_instance, |data| {
            let desc = bytemuck::from_bytes::<GpuMarchingCubesDescription>(&data);
            indices_counter = desc.indices_counter;
            c_sub_count = [desc.chunk_sub_count[0] as usize, desc.chunk_sub_count[1] as usize, desc.chunk_sub_count[2] as usize];
        });
        debug!("Chunk {:?} generated with {} indices.", index, indices_counter);

        // Warn if the indices counter is too high
        let vertex_buffer_size = std::mem::size_of::<WVertex>() * 3 * 5 * c_sub_count[0] * c_sub_count[1] * c_sub_count[2];
        if std::mem::size_of::<WVertex>() * (indices_counter as usize) > vertex_buffer_size {
            error!("In the marching cubes algorithm, there is too much vertices overflowing the vertices buffer. The buffer size is {} and the indices counter is {}.", vertex_buffer_size, indices_counter);
            continue;
        }

        // Update the chunk
        chunk.indices_counter = indices_counter;
        chunk.generated = true;

        // Add the chunk to the active chunks
        new_chunks.push((*index, chunk.clone()));
    }

    // Add the new chunks to the active chunks
    for (index, chunk) in new_chunks {
        handler.loading_chunks.remove(&index);
        handler.active_chunks.insert(index, chunk);
    }
}
}
