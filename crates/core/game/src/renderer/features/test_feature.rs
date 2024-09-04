use bevy::prelude::*;
use wde_wgpu::{command_buffer::{Color, WCommandBuffer, LoadOp, Operations, StoreOp}, instance::WRenderInstance};

use crate::{renderer::{extract_macros::*, render_assets::RenderAssets, Extract, Render, RenderSet, SwapchainFrame}, scene::{components::TestComponent, resources::{GpuTexture, Texture}}};


pub struct TestFeature;
impl Plugin for TestFeature {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TestElementsRender>()
            .add_systems(Extract, extract_render_test)
            .add_systems(Render, render_test.in_set(RenderSet::Render));
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

fn render_test(render_instance: Res<WRenderInstance<'static>>, swapchain_frame: Res<SwapchainFrame>, test_elements: Res<TestElementsRender>, textures: Res<RenderAssets<GpuTexture>>) {
    let render_instance = render_instance.data.lock().unwrap();
    let swapchain_frame = swapchain_frame.data.as_ref().unwrap();
    let mut command_buffer = WCommandBuffer::new(&render_instance, "dummy");

    {
        let mut _render_pass = command_buffer.create_render_pass(
            "dummy", &swapchain_frame.view,
            Some(Operations {
                load: LoadOp::Clear(Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 }),
                store: StoreOp::Store,
            }),
            None);

        // Render
        for heightmap in test_elements.heightmaps.iter() {
            if let Some(_heightmap) = textures.get(heightmap) {
                warn!("Rendering heightmap!");
            } else {
                // warn!("Heightmap texture not loaded yet");
            }
        }

        // Dummy texture display
        // render_pass.set_vertex_buffer(0, &quad_vertex_buffer);
        // render_pass.set_bind_group(0, &texture_test_bind_group);
        // render_pass.set_pipeline(&texture_test_pipeline).unwrap();
        // render_pass.draw(0..6, 0..1);
    }

    command_buffer.submit(&render_instance);
}
