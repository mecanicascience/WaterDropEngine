use bevy::prelude::*;
use wde_render::{assets::{render_assets::RenderAssets, GpuBuffer, GpuMesh, Mesh}, core::{Render, RenderApp, RenderSet, SwapchainFrame}, features::{CameraFeatureBuffer, CameraFeatureLayout}, pipelines::{CachedPipelineIndex, CachedPipelineStatus, PipelineManager, RenderPipelineDescriptor}};
use wde_wgpu::{bind_group::BindGroup, command_buffer::{Color, LoadOp, Operations, StoreOp, WCommandBuffer}, instance::WRenderInstance, vertex::WVertex};

#[derive(Resource)]
pub struct MeshPipeline {
    pub index: CachedPipelineIndex
}
impl FromWorld for MeshPipeline {
    fn from_world(world: &mut World) -> Self {
        // Get the camera layout
        let layout = &world.get_resource::<CameraFeatureLayout>().unwrap().layout;

        // Create the pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: "mesh",
            vert: Some(world.load_asset("mesh/vert.wgsl")),
            frag: Some(world.load_asset("mesh/frag.wgsl")),
            depth_stencil: false,
            bind_group_layouts: vec![layout.clone()],
            cull_mode: None,
            ..Default::default()
        };
        let cached_index = world.get_resource_mut::<PipelineManager>().unwrap().create_render_pipeline(pipeline_desc);
        MeshPipeline { index: cached_index }
    }
}

#[derive(Resource, Default)]
struct MeshHandler {
    mesh: Handle<Mesh>
}

pub struct MeshFeature;
impl Plugin for MeshFeature {
    fn build(&self, app: &mut App) {
        // Create the 2d quad mesh
        let post_process_mesh: Handle<Mesh> = app.world_mut().add_asset(Mesh {
            label: "quad".to_string(),
            vertices: vec![
                WVertex { position: [-1.0, 1.0, 0.0], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [-1.0, -1.0, 0.0], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [1.0, -1.0, 0.0], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [1.0, 1.0, 0.0], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        });
        
        app.get_sub_app_mut(RenderApp).unwrap()
            .add_systems(Render, render_texture.in_set(RenderSet::Render))
            .init_resource::<MeshPipeline>()
            .insert_resource(MeshHandler { mesh: post_process_mesh });
    }
}

fn render_texture(
    (render_instance, swapchain_frame, pipeline_manager): (
        Res<WRenderInstance<'static>>, Res<SwapchainFrame>,  Res<PipelineManager>
    ),
    (camera_buffer, camera_layout) : (
        Res<CameraFeatureBuffer>, Res<CameraFeatureLayout>
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
            Some(mesh),
            Some(camera_buffer),
        ) = (
            pipeline_manager.get_pipeline(mesh_pipeline.index),
            meshes.get(&mesh_handler.mesh),
            buffers.get(&camera_buffer.buffer),
        ) {
            // Set the camera bind group
            let bind_group = BindGroup::build("camera", &render_instance, &camera_layout.layout_built, &vec![
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
