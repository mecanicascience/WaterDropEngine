use bevy::prelude::*;
use wde_wgpu::{command_buffer::{Color, LoadOp, Operations, StoreOp, WCommandBuffer}, instance::WRenderInstance, vertex::WVertex};

use crate::{renderer::{extract_macros::*, render_assets::RenderAssets, CachedPipelineIndex, CachedPipelineStatus, Extract, PipelineManager, Render, RenderApp, RenderPipelineDescriptor, RenderSet, SwapchainFrame}, scene::{components::TestComponent, resources::{GpuMesh, GpuTexture, Mesh, Texture}}};


#[derive(Resource)]
pub struct TestPipeline {
    pub index: CachedPipelineIndex
}
impl FromWorld for TestPipeline {
    fn from_world(world: &mut World) -> Self {
        // Create the pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: "TestPipeline",
            vert: Some(world.load_asset("test/test.vert.wgsl")),
            frag: Some(world.load_asset("test/test.frag.wgsl")),
            depth_stencil: false,
            ..Default::default()
        };
        let cached_index = world.get_resource_mut::<PipelineManager>().unwrap().create_render_pipeline(pipeline_desc);
        TestPipeline { index: cached_index }
    }
}

#[derive(Resource, Default)]
pub struct TestPipelineMesh {
    pub mesh: Handle<Mesh>,
}

pub struct TestFeature;
impl Plugin for TestFeature {
    fn build(&self, app: &mut App) {
        {
            let render_app = app.get_sub_app_mut(RenderApp).unwrap();
            render_app
                .init_resource::<TestElementsRender>()
                .add_systems(Extract, extract_render_test)
                .add_systems(Render, render_test.in_set(RenderSet::Render));
        }

        // Create the post process mesh
        let post_process_mesh: Handle<Mesh> = app.world_mut().add_asset(Mesh {
            label: "PostProcessQuad".to_string(),
            vertices: vec![
                WVertex { position: [-1.0, 1.0, 0.0], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [-1.0, -1.0, 0.0], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [1.0, -1.0, 0.0], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [1.0, 1.0, 0.0], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        });
        
        // Add resources
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .init_resource::<TestPipeline>()
            .insert_resource(TestPipelineMesh { mesh: post_process_mesh });
    }
}




#[derive(Resource, Default)]
pub struct TestElementsRender {
    pub heightmaps: Vec<Handle<Texture>>,
}

fn extract_render_test(test_elements: ExtractWorld<Query<Ref<TestComponent>>>, mut test_elements_render: ResMut<TestElementsRender>) {
    test_elements_render.heightmaps.clear();
    for test_element in test_elements.iter() {
        // Add the heightmap to the render list
        test_elements_render.heightmaps.push(test_element.heightmap.clone());
    }
}

fn render_test(
    (render_instance, swapchain_frame, pipeline_manager): (
        Res<WRenderInstance<'static>>, Res<SwapchainFrame>,  Res<PipelineManager>
    ),
    (textures, meshes): (
        Res<RenderAssets<GpuTexture>>, Res<RenderAssets<GpuMesh>>
    ),
    (test_elements, texture_test_pipeline, test_pipeline_mesh): (
        Res<TestElementsRender>, Res<TestPipeline>, Res<TestPipelineMesh>,
    )
) {
    let render_instance = render_instance.data.lock().unwrap();
    let swapchain_frame = swapchain_frame.data.as_ref().unwrap();
    let mut command_buffer = WCommandBuffer::new(&render_instance, "dummy");

    {
        let mut render_pass = command_buffer.create_render_pass(
            "dummy", &swapchain_frame.view,
            Some(Operations {
                load: LoadOp::Clear(Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }),
                store: StoreOp::Store,
            }),
            None);

        // Render
        for heightmap in test_elements.heightmaps.iter() {
            if let Some(_heightmap) = textures.get(heightmap) {
                // warn!("Rendering heightmap!");
            } else {
                // warn!("Heightmap texture not loaded yet");
            }
        }

        // Dummy texture display
        if let CachedPipelineStatus::Ok(pipeline) = pipeline_manager.get_pipeline(texture_test_pipeline.index) {
            match render_pass.set_pipeline(pipeline) {
                Ok(_) => {},
                Err(e) => {
                    error!("Failed to set pipeline: {:?}", e);
                    return;
                }
            }

            // Get the mesh
            if let Some(mesh) = meshes.get(&test_pipeline_mesh.mesh) {
                render_pass.set_vertex_buffer(0, &mesh.vertex_buffer);
                render_pass.set_index_buffer(&mesh.index_buffer);

                match render_pass.draw_indexed(0..mesh.index_count, 0..1) {
                    Ok(_) => {},
                    Err(e) => {
                        error!("Failed to draw: {:?}", e);
                        return;
                    }
                };
            }
        };
    }

    command_buffer.submit(&render_instance);
}
