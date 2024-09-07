use bevy::prelude::*;
use wde_render::{assets::{GpuBuffer, GpuMesh, Mesh, RenderAssets}, core::{extract_macros::ExtractWorld, Extract, Render, RenderApp, RenderSet, SwapchainFrame}, features::{CameraFeatureBuffer, CameraFeatureLayout}, pipelines::{CachedPipelineIndex, CachedPipelineStatus, PipelineManager, RenderPipelineDescriptor}};
use wde_wgpu::{bind_group::BindGroup, command_buffer::{Color, LoadOp, Operations, StoreOp, WCommandBuffer}, instance::WRenderInstance};

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

pub struct MeshFeature;
impl Plugin for MeshFeature {
    fn build(&self, app: &mut App) {
        app.get_sub_app_mut(RenderApp).unwrap()
            .add_systems(Extract, extract_meshes)
            .add_systems(Render, render.in_set(RenderSet::Render))
            .init_resource::<MeshPipeline>();
    }
}

pub struct RenderMeshPassEntity {
    pub transform: Transform,
    pub mesh: Handle<Mesh>,
}

#[derive(Resource)]
pub struct RenderMeshPass {
    pub entities: Vec<RenderMeshPassEntity>
}

fn extract_meshes(mut commands: Commands, entities : ExtractWorld<Query<(&Transform, &Handle<Mesh>)>>) {
    let mut render_entities = Vec::with_capacity(entities.iter().count());
    for (transform, draw) in entities.iter() {
        render_entities.push(RenderMeshPassEntity {
            transform: *transform,
            mesh: draw.clone()
        });
    }
    commands.insert_resource(RenderMeshPass { entities: render_entities });
}

fn render(
    (render_instance, swapchain_frame, pipeline_manager): (
        Res<WRenderInstance<'static>>, Res<SwapchainFrame>,  Res<PipelineManager>
    ),
    (camera_buffer, camera_layout) : (
        Res<CameraFeatureBuffer>, Res<CameraFeatureLayout>
    ),
    (meshes, buffers) : (
        Res<RenderAssets<GpuMesh>>, Res<RenderAssets<GpuBuffer>>
    ),
    (mesh_pipeline, render_mesh_pass): (
        Res<MeshPipeline>, Res<RenderMeshPass>
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
            Some(camera_buffer),
        ) = (
            pipeline_manager.get_pipeline(mesh_pipeline.index),
            buffers.get(&camera_buffer.buffer),
        ) {
            // Set the camera bind group
            let bind_group = BindGroup::build("camera", &render_instance, &camera_layout.layout_built, &vec![
                BindGroup::buffer(0, &camera_buffer.buffer)
            ]);
            render_pass.set_bind_group(0, &bind_group);

            // Set the pipeline
            if render_pass.set_pipeline(pipeline).is_ok() {
                for entity in render_mesh_pass.entities.iter() {
                    if let Some(mesh) = meshes.get(&entity.mesh) {
                        // Set the transform
                        // TODO

                        // Set the mesh buffers
                        render_pass.set_vertex_buffer(0, &mesh.vertex_buffer);
                        render_pass.set_index_buffer(&mesh.index_buffer);

                        // Draw the mesh
                        match render_pass.draw_indexed(0..mesh.index_count, 0..1) {
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
