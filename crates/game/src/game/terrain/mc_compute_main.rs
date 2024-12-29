use bevy::prelude::*;
use wde_render::{assets::Buffer, core::{extract_macros::ExtractWorld, DeviceLimits}};
use wde_wgpu::buffer::BufferUsage;

use super::mc_chunk::{MC_MAX_POINTS, MC_MAX_TRIANGLES};

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
    pub triangles_cpu: Option<Handle<Buffer>>,
    pub triangles_gpu: Option<Handle<Buffer>>
}
#[derive(Resource, Default)]
pub struct MCComputeHandlerGPU {
    // Buffers
    pub desc_cpu: Option<Handle<Buffer>>,
    pub desc_gpu: Option<Handle<Buffer>>,
    pub points_cpu: Option<Handle<Buffer>>,
    pub triangles_cpu: Option<Handle<Buffer>>,
    pub triangles_gpu: Option<Handle<Buffer>>
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
            size: std::cmp::min(std::mem::size_of::<[f32; 4]>() * MC_MAX_POINTS as usize, max_buffer_size),
            usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            content: None
        };
        let triangles_cpu = Buffer {
            label: "marching-cubes-triangles-cpu".to_string(),
            size: std::cmp::min(std::mem::size_of::<[f32; 12]>() * MC_MAX_TRIANGLES as usize, max_buffer_size),
            usage: BufferUsage::MAP_READ | BufferUsage::COPY_DST,
            content: None
        };
        let triangles_gpu = Buffer {
            label: "marching-cubes-triangles-gpu".to_string(),
            size: std::cmp::min(std::mem::size_of::<[f32; 12]>() * MC_MAX_TRIANGLES as usize, max_buffer_size),
            usage: BufferUsage::STORAGE | BufferUsage::COPY_SRC,
            content: None
        };

        // Create the handler
        commands.insert_resource(MCComputeHandler {
            desc_cpu: Some(asset_server.add(desc_cpu)),
            desc_gpu: Some(asset_server.add(desc_gpu)),
            points_cpu: Some(asset_server.add(points_cpu)),
            triangles_cpu: Some(asset_server.add(triangles_cpu)),
            triangles_gpu: Some(asset_server.add(triangles_gpu))
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
            handler_render.triangles_cpu = handler_update.triangles_cpu.clone();
            handler_render.triangles_gpu = handler_update.triangles_gpu.clone();
        }
    }
}
