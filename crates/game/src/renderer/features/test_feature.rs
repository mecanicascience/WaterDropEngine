use bevy::prelude::*;
use wde_wgpu::{bind_group::{BindGroup, BindGroupLayout, WgpuBindGroupLayout}, command_buffer::{Color, LoadOp, Operations, StoreOp, WCommandBuffer}, instance::WRenderInstance, render_pipeline::ShaderStages, vertex::WVertex};

use crate::{renderer::{extract_macros::*, render_assets::RenderAssets, CachedPipelineIndex, CachedPipelineStatus, Extract, PipelineManager, Render, RenderApp, RenderPipelineDescriptor, RenderSet, SwapchainFrame}, scene::{components::TestComponent, resources::{GpuMesh, GpuTexture, Mesh, Texture}}};


#[derive(Resource)]
pub struct TestPipeline {
    pub index: CachedPipelineIndex,
    pub test_layout: BindGroupLayout,
    pub test_layout_built: Option<WgpuBindGroupLayout>,
}
impl TestPipeline {
    fn build(mut pipeline: ResMut<TestPipeline>, render_instance: Res<WRenderInstance<'static>>) {
        pipeline.test_layout_built = Some(pipeline.test_layout.build(&render_instance.data.lock().unwrap()));
    }
}
impl FromWorld for TestPipeline {
    fn from_world(world: &mut World) -> Self {
        // Create the test layout
        let test_layout = BindGroupLayout::new("TestPipeline", |builder| {
            builder.add_texture(0, ShaderStages::FRAGMENT);
        });

        // Create the pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: "TestPipeline",
            vert: Some(world.load_asset("test/test.vert.wgsl")),
            frag: Some(world.load_asset("test/test.frag.wgsl")),
            depth_stencil: false,
            bind_group_layouts: vec![test_layout.clone()],
            ..Default::default()
        };
        let cached_index = world.get_resource_mut::<PipelineManager>().unwrap().create_render_pipeline(pipeline_desc);
        TestPipeline { index: cached_index, test_layout, test_layout_built: None }
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
                .add_systems(Render, TestPipeline::build.in_set(RenderSet::Prepare))
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
    // Render the test elements
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

        if !test_elements.heightmaps.is_empty() {
            // Dummy texture display
            if let (
                CachedPipelineStatus::Ok(pipeline),
                Some(layout),
                Some(mesh),
                Some(heightmap)
            ) = (
                pipeline_manager.get_pipeline(texture_test_pipeline.index),
                &texture_test_pipeline.test_layout_built,
                meshes.get(&test_pipeline_mesh.mesh),
                textures.get(&test_elements.heightmaps[0])
            ) {
                // Set the pipeline
                if render_pass.set_pipeline(pipeline).is_ok() {
                    // Get the mesh
                    render_pass.set_vertex_buffer(0, &mesh.vertex_buffer);
                    render_pass.set_index_buffer(&mesh.index_buffer);

                    // Set bind group
                    let bind_group = BindGroup::build("TestPipeline", &render_instance, layout, &vec![
                        BindGroup::texture_view(0, &heightmap.texture),
                        BindGroup::texture_sampler(1, &heightmap.texture)
                    ]);
                    render_pass.set_bind_group(0, &bind_group);

                    match render_pass.draw_indexed(0..mesh.index_count, 0..1) {
                        Ok(_) => {},
                        Err(e) => {
                            error!("Failed to draw: {:?}", e);
                        }
                    };
                } else {
                    error!("Failed to set pipeline");
                }
            }
        }
    }

    command_buffer.submit(&render_instance);
}
