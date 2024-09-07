use bevy::prelude::*;
use wde_wgpu::{bind_group::{BindGroupLayout, WgpuBindGroupLayout}, buffer::{BufferBindingType, BufferUsage}, instance::WRenderInstance, render_pipeline::WShaderStages};

use crate::{assets::{RenderAssets, Buffer, GpuBuffer}, components::{CameraUniform, CameraView}, core::{extract_macros::ExtractWorld, Extract, Render, RenderApp, RenderSet}};

/// Struct to hold the camera uniform layout description.
#[derive(Resource)]
pub struct CameraFeatureLayout {
    pub layout: BindGroupLayout,
    pub layout_built: WgpuBindGroupLayout,
}
impl FromWorld for CameraFeatureLayout {
    fn from_world(world: &mut World) -> Self {
        let render_instance = world.get_resource::<WRenderInstance<'static>>().unwrap();

        // Create the camera layout
        let layout = BindGroupLayout::new("camera", |builder| {
            builder.add_buffer(0, WShaderStages::VERTEX, BufferBindingType::Uniform);
        });
        let layout_built = layout.build(&render_instance.data.lock().unwrap());
        
        CameraFeatureLayout { layout, layout_built }
    }
}

/// Struct to hold the camera uniform buffer.
#[derive(Resource, Default)]
pub struct CameraFeatureBuffer {
    pub buffer: Handle<Buffer>,
}

pub struct CameraFeature;
impl Plugin for CameraFeature {
    fn build(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp).unwrap()
            .add_systems(Extract, extract)
            .add_systems(Render, update_buffer.in_set(RenderSet::Prepare))
            .init_resource::<CameraFeatureLayout>()
            .init_resource::<CameraUniform>();
    }

    fn finish(&self, app: &mut App) {
        // Create the camera buffer (need that the assets have been initialized)
        let buffer: Handle<Buffer> = app.world_mut().add_asset(Buffer {
            label: "camera".to_string(),
            size: std::mem::size_of::<CameraUniform>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            content: None,
        });
        
        // Add resources
        app.get_sub_app_mut(RenderApp).unwrap()
            .insert_resource(CameraFeatureBuffer { buffer });
    }
}


// Extract the texture handle every frame
fn extract(
    (cameras, mut camera_uniform): (
        ExtractWorld<Query<(&Transform, &CameraView)>>, ResMut<CameraUniform>
    ), window: ExtractWorld<Query<&Window>>)
{
    if let (
        Ok((transform, view)), Ok(window)
    ) = (cameras.get_single(), window.get_single()) {
        // Update the camera uniform
        let aspect_ratio = window.width() / window.height();
        camera_uniform.world_to_ndc = CameraUniform::get_world_to_ndc(transform, view, aspect_ratio).to_cols_array_2d();
    }
}

// Update the camera buffer
fn update_buffer(
    (render_instance, camera_uniform, camera_buffer): (
        Res<WRenderInstance<'static>>, Res<CameraUniform>, Res<CameraFeatureBuffer>
    ),
    mut buffers: ResMut<RenderAssets<GpuBuffer>>
) {
    // Update the camera buffer
    if let Some(camera_buffer) = buffers.get_mut(&camera_buffer.buffer) {
        let render_instance = render_instance.data.lock().unwrap();
        camera_buffer.buffer.write(&render_instance, bytemuck::cast_slice(&[camera_uniform.to_owned()]), 0);
    }
}
