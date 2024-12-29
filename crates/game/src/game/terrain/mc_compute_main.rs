use bevy::prelude::*;
use wde_render::{assets::Buffer, core::{extract_macros::ExtractWorld, DeviceLimits}};
use wde_wgpu::buffer::BufferUsage;

use crate::terrain::mc_chunk::MC_MAX_SUB_COUNT;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug, Default)]
/**
 * Description of a chunk for the compute shader used in the marching cubes algorithm.
 */
pub struct GpuMarchingCubesDescription {
    pub translation:       [f32; 4], // Translation in world space of the current chunk (x, y, z, 0)
    pub chunk_length:      [f32; 4], // Length of the chunk (x, y, z, 0)
    pub chunk_sub_count:   [u32; 4], // Number of sub-chunks (x, y, z, 0)
    pub triangles_counter: u32,      // Counter of the triangles
    pub iso_level:         f32,      // Iso level
    pub padding:           [u32; 2]  // Padding
}

#[derive(Resource, Default)]
pub struct MCComputeHandler {
    // Buffers
    pub desc_cpu: Option<Handle<Buffer>>,
    pub desc_gpu: Option<Handle<Buffer>>,
    pub points_cpu: Option<Handle<Buffer>>,
    pub vertices_cpu: Option<Handle<Buffer>>,
    pub vertices_gpu: Option<Handle<Buffer>>
}
#[derive(Resource, Default)]
pub struct MCComputeHandlerGPU {
    // Buffers
    pub desc_cpu: Option<Handle<Buffer>>,
    pub desc_gpu: Option<Handle<Buffer>>,
    pub points_cpu: Option<Handle<Buffer>>,
    pub vertices_cpu: Option<Handle<Buffer>>,
    pub vertices_gpu: Option<Handle<Buffer>>
}
impl MCComputeHandler {
    /** Creates the asset handler buffers and instance (runs once). */
    pub fn init(
        asset_server: Res<AssetServer>,
        device_limits: Res<DeviceLimits>,
        mut commands: Commands
    ) {
        // Create the buffers
        let max_buffer_size = device_limits.0.max_storage_buffer_binding_size as usize;
        let desc_cpu = Buffer {
            label: "marching-cubes-desc-cpu".to_string(),
            size: std::mem::size_of::<GpuMarchingCubesDescription>(),
            usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            content: None
        };
        let desc_gpu = Buffer {
            label: "marching-cubes-desc-gpu".to_string(),
            size: std::mem::size_of::<GpuMarchingCubesDescription>(),
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST | BufferUsage::COPY_SRC,
            content: None
        };
        let points_cpu = Buffer {
            label: "marching-cubes-points-cpu".to_string(),
            size: std::cmp::min(std::mem::size_of::<[f32; 4]>() * (MC_MAX_SUB_COUNT[0] * MC_MAX_SUB_COUNT[1] * MC_MAX_SUB_COUNT[2]) as usize, max_buffer_size),
            usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            content: None
        };
        let vertices_cpu = Buffer {
            label: "marching-cubes-vertices-cpu".to_string(),
            size: std::cmp::min(std::mem::size_of::<[f32; 12]>() * 5 * (MC_MAX_SUB_COUNT[0] * MC_MAX_SUB_COUNT[1] * MC_MAX_SUB_COUNT[2]) as usize, max_buffer_size),
            usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            content: None
        };
        let vertices_gpu = Buffer {
            label: "marching-cubes-vertices-gpu".to_string(),
            size: std::cmp::min(std::mem::size_of::<[f32; 12]>() * 5 * (MC_MAX_SUB_COUNT[0] * MC_MAX_SUB_COUNT[1] * MC_MAX_SUB_COUNT[2]) as usize, max_buffer_size),
            usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC,
            content: None
        };

        // Create the handler
        commands.insert_resource(MCComputeHandler {
            desc_cpu: Some(asset_server.add(desc_cpu)),
            desc_gpu: Some(asset_server.add(desc_gpu)),
            points_cpu: Some(asset_server.add(points_cpu)),
            vertices_cpu: Some(asset_server.add(vertices_cpu)),
            vertices_gpu: Some(asset_server.add(vertices_gpu))
        });
    }


    /** Extract the new chunks in the main thread and add them to the loading chunks in the render thread. */
    pub fn extract(
        handler_update: ExtractWorld<Res<MCComputeHandler>>,
        mut handler_render: ResMut<MCComputeHandlerGPU>,
    ) {
        // Extract the buffers if they are not already extracted
        if handler_render.desc_cpu.is_none() && handler_update.desc_cpu.is_some() {
            handler_render.desc_cpu = handler_update.desc_cpu.clone();
            handler_render.desc_gpu = handler_update.desc_gpu.clone();
            handler_render.points_cpu = handler_update.points_cpu.clone();
            handler_render.vertices_cpu = handler_update.vertices_cpu.clone();
            handler_render.vertices_gpu = handler_update.vertices_gpu.clone();
        }
    }
}
