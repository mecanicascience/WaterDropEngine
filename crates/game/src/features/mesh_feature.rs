use bevy::prelude::*;
use wde_render::{assets::{render_assets::RenderAssets, Buffer, GpuBuffer, GpuMesh, Mesh}, components::{CameraUniform, CameraViewComponent, TransformComponent}, core::{extract_macros::ExtractWorld, Extract, Render, RenderApp, RenderSet, SwapchainFrame}, pipelines::{CachedPipelineIndex, CachedPipelineStatus, PipelineManager, RenderPipelineDescriptor}};
use wde_wgpu::{bind_group::{BindGroup, BindGroupLayout, WgpuBindGroupLayout}, buffer::{BufferBindingType, BufferUsage}, command_buffer::{Color, LoadOp, Operations, StoreOp, WCommandBuffer}, instance::WRenderInstance, render_pipeline::ShaderStages, vertex::WVertex};

#[derive(Resource)]
pub struct MeshPipeline {
    pub index: CachedPipelineIndex,
    pub camera_layout: BindGroupLayout,
    pub camera_layout_built: Option<WgpuBindGroupLayout>,
}
impl MeshPipeline {
    fn build(mut pipeline: ResMut<MeshPipeline>, render_instance: Res<WRenderInstance<'static>>) {
        pipeline.camera_layout_built = Some(pipeline.camera_layout.build(&render_instance.data.lock().unwrap()));
    }
}
impl FromWorld for MeshPipeline {
    fn from_world(world: &mut World) -> Self {
        // Create the camera layout
        let layout = BindGroupLayout::new("camera", |builder| {
            builder.add_buffer(0, ShaderStages::VERTEX, BufferBindingType::Uniform);
        });

        // Create the pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: "mesh",
            vert: Some(world.load_asset("mesh/vert.wgsl")),
            frag: Some(world.load_asset("mesh/frag.wgsl")),
            depth_stencil: false,
            bind_group_layouts: vec![layout.clone()],
            ..Default::default()
        };
        let cached_index = world.get_resource_mut::<PipelineManager>().unwrap().create_render_pipeline(pipeline_desc);
        MeshPipeline { index: cached_index, camera_layout: layout, camera_layout_built: None }
    }
}

#[derive(Resource, Default)]
struct MeshHandler {
    mesh: Handle<Mesh>,
    camera_buffer: Handle<Buffer>,
}


pub struct MeshFeature;
impl Plugin for MeshFeature {
    fn build(&self, app: &mut App) {
        {
            let render_app = app.get_sub_app_mut(RenderApp).unwrap();
            render_app
                .add_systems(Extract, extract_camera)
                .add_systems(Render, MeshPipeline::build.in_set(RenderSet::Prepare).run_if(run_once()))
                .add_systems(Render, update_camera_buffer.in_set(RenderSet::Prepare))
                .add_systems(Render, render_texture.in_set(RenderSet::Render))
                .init_resource::<CameraUniform>();
        }

        // Create the 2d quad mesh
        let post_process_mesh: Handle<Mesh> = app.world_mut().add_asset(Mesh {
            label: "post-process-quad".to_string(),
            vertices: vec![
                WVertex { position: [-1.0, 1.0, 0.0], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [-1.0, -1.0, 0.0], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [1.0, -1.0, 0.0], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [1.0, 1.0, 0.0], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        });

        // Create the camera buffer
        let camera_buffer: Handle<Buffer> = app.world_mut().add_asset(Buffer {
            label: "camera-buffer".to_string(),
            size: std::mem::size_of::<CameraUniform>(),
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            content: None,
        });
        
        // Add resources
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .init_resource::<MeshPipeline>()
            .insert_resource(MeshHandler { mesh: post_process_mesh, camera_buffer });
    }
}


// Extract the texture handle every frame
fn extract_camera(cameras: ExtractWorld<Query<(&TransformComponent, &CameraViewComponent)>>, mut camera_uniforms: ResMut<CameraUniform>) {
    if let Some((transform, view)) = cameras.iter().next() {
        // Update the camera uniforms
        camera_uniforms.world_to_ndc = CameraUniform::get_world_to_ndc(transform, view).to_cols_array_2d();
    }
}

// Update the camera buffer
fn update_camera_buffer(
    (render_instance, camera_uniform, mesh_handler): (
        Res<WRenderInstance<'static>>, Res<CameraUniform>, Res<MeshHandler>
    ),
    mut buffers: ResMut<RenderAssets<GpuBuffer>>
) {
    // Update the camera buffer
    if let Some(camera_buffer) = buffers.get_mut(&mesh_handler.camera_buffer) {
        let render_instance = render_instance.data.lock().unwrap();
        camera_buffer.buffer.write(&render_instance, bytemuck::cast_slice(&[camera_uniform.to_owned()]), 0);
    }
}

fn render_texture(
    (render_instance, swapchain_frame, pipeline_manager): (
        Res<WRenderInstance<'static>>, Res<SwapchainFrame>,  Res<PipelineManager>
    ),
    (meshes, buffers) : (
        Res<RenderAssets<GpuMesh>>, Res<RenderAssets<GpuBuffer>>
    ),
    (mesh_pipeline, mesh_handler): (
        Res<MeshPipeline>, Res<MeshHandler>,
    )
) {
    // Render the texture
    let render_instance = render_instance.data.lock().unwrap();
    let swapchain_frame = swapchain_frame.data.as_ref().unwrap();
    let mut command_buffer = WCommandBuffer::new(&render_instance, "mesh");

    {
        let mut render_pass = command_buffer.create_render_pass(
            "mesh", &swapchain_frame.view,
            Some(Operations {
                load: LoadOp::Clear(Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                store: StoreOp::Store,
            }),
            None);

        // Render the mesh
        if let (
            CachedPipelineStatus::Ok(pipeline),
            Some(camera_layout),
            Some(mesh),
            Some(camera_buffer),
        ) = (
            pipeline_manager.get_pipeline(mesh_pipeline.index),
            mesh_pipeline.camera_layout_built.as_ref(),
            meshes.get(&mesh_handler.mesh),
            buffers.get(&mesh_handler.camera_buffer),
        ) {
            // Set the camera bind group
            let bind_group = BindGroup::build("camera", &render_instance, camera_layout, &vec![
                BindGroup::buffer(0, &camera_buffer.buffer)
            ]);
            render_pass.set_bind_group(0, &bind_group);

            // Set the pipeline
            if render_pass.set_pipeline(pipeline).is_ok() {
                // Get the mesh
                render_pass.set_vertex_buffer(0, &mesh.vertex_buffer);
                render_pass.set_index_buffer(&mesh.index_buffer);

                // Draw the mesh
                match render_pass.draw_indexed(0..mesh.index_count, 0..1) {
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
