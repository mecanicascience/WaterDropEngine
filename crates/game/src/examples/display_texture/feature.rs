use bevy::prelude::*;
use wde_render::{assets::{GpuMesh, GpuTexture, MeshAsset, ModelBoundingBox, RenderAssets, Texture}, core::{extract_macros::ExtractWorld, Extract, Render, RenderApp, RenderSet, SwapchainFrame}, pipelines::{CachedPipelineIndex, CachedPipelineStatus, PipelineManager, RenderPipelineDescriptor}};
use wde_wgpu::{bind_group::{BindGroup, BindGroupLayout, WgpuBindGroup, WgpuBindGroupLayout}, command_buffer::{RenderPassBuilder, RenderPassColorAttachment, WCommandBuffer}, instance::WRenderInstance, render_pipeline::WShaderStages, vertex::WVertex};

use super::component::DisplayTextureComponent;

#[derive(Resource)]
pub struct DisplayTexturePipeline {
    pub index: CachedPipelineIndex,
    pub layout: BindGroupLayout,
    pub layout_built: Option<WgpuBindGroupLayout>,
    pub bind_group: Option<WgpuBindGroup>,
}
impl DisplayTexturePipeline {
    fn build(mut pipeline: ResMut<DisplayTexturePipeline>, render_instance: Res<WRenderInstance<'static>>) {
        pipeline.layout_built = Some(pipeline.layout.build(&render_instance.data.read().unwrap()));
    }
}
impl FromWorld for DisplayTexturePipeline {
    fn from_world(world: &mut World) -> Self {
        // Create the layout of the bind group at binding 0
        let layout = BindGroupLayout::new("display-texture", |builder| {
            // Set the texture view and sampler
            builder.add_texture_view(0, WShaderStages::FRAGMENT);
            builder.add_texture_sampler(1, WShaderStages::FRAGMENT);
        });

        // Create the pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: "display-texture",
            vert: Some(world.load_asset("examples/display_texture/vert.wgsl")),
            frag: Some(world.load_asset("examples/display_texture/frag.wgsl")),
            bind_group_layouts: vec![layout.clone()],
            ..Default::default()
        };
        let cached_index = world.get_resource_mut::<PipelineManager>().unwrap().create_render_pipeline(pipeline_desc);
        DisplayTexturePipeline { index: cached_index, layout, layout_built: None, bind_group: None }
    }
}



#[derive(Resource, Default)]
pub struct DisplayTextureMesh {
    pub mesh: Handle<MeshAsset>,
}

#[derive(Resource, Default)]
pub struct DisplayTextureHolder {
    pub texture: Option<Handle<Texture>>,
}

pub struct DisplayTextureFeature;
impl Plugin for DisplayTextureFeature {
    fn build(&self, app: &mut App) {
        {
            let render_app = app.get_sub_app_mut(RenderApp).unwrap();
            render_app
                .init_resource::<DisplayTextureHolder>()
                .add_systems(Extract, extract_texture)
                .add_systems(Render, DisplayTexturePipeline::build.in_set(RenderSet::Prepare).run_if(run_once))
                .add_systems(Render, prepare_texture_bind_group.in_set(RenderSet::BindGroups))
                .add_systems(Render, render_texture.in_set(RenderSet::Render));
        }

        // Create the 2d quad mesh
        let post_process_mesh: Handle<MeshAsset> = app.world_mut().add_asset(MeshAsset {
            label: "PostProcessQuad".to_string(),
            vertices: vec![
                WVertex { position: [-1.0, 1.0, 0.0], uv: [0.0, 1.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [-1.0, -1.0, 0.0], uv: [0.0, 0.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [1.0, -1.0, 0.0], uv: [1.0, 0.0], normal: [0.0, 0.0, 0.0] },
                WVertex { position: [1.0, 1.0, 0.0], uv: [1.0, 1.0], normal: [0.0, 0.0, 0.0] },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            bounding_box: ModelBoundingBox {
                min: Vec3::new(-1.0, -1.0, 0.0),
                max: Vec3::new(1.0, 1.0, 0.0),
            },
        });
        
        // Add resources
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        render_app
            .init_resource::<DisplayTexturePipeline>()
            .insert_resource(DisplayTextureMesh { mesh: post_process_mesh });
    }
}



// Extract the texture handle every frame
fn extract_texture(display_texture_cpus: ExtractWorld<Query<Ref<DisplayTextureComponent>>>, mut display_texture_holder: ResMut<DisplayTextureHolder>) {
    display_texture_holder.texture = None;
    if let Some(display_texture_cpu) = display_texture_cpus.iter().next() {
        display_texture_holder.texture = Some(display_texture_cpu.texture.clone());
    }
}

// Prepare the bind group
fn prepare_texture_bind_group(
    (render_instance, display_texture_holder, mut texture_pipeline): (
        Res<WRenderInstance<'static>>, Res<DisplayTextureHolder>, ResMut<DisplayTexturePipeline>
    ),
    display_textures: Res<RenderAssets<GpuTexture>>
) {
    // Check if bind group already exists
    if texture_pipeline.bind_group.is_some() {
        return;
    }

    if let (Some(texture), Some(pipeline_layout)) = (
        display_textures.get(display_texture_holder.texture.as_ref().unwrap()),
        &texture_pipeline.layout_built
    ) {
        let render_instance = render_instance.data.read().unwrap();
        let bind_group = BindGroup::build("display-texture", &render_instance, pipeline_layout, &vec![
            BindGroup::texture_view(0, &texture.texture),
            BindGroup::texture_sampler(1, &texture.texture)
        ]);
        texture_pipeline.bind_group = Some(bind_group);
    }
}

fn render_texture(
    (render_instance, swapchain_frame, pipeline_manager): (
        Res<WRenderInstance<'static>>, Res<SwapchainFrame>,  Res<PipelineManager>
    ),
    meshes: Res<RenderAssets<GpuMesh>>,
    (display_texture_holders, texture_test_pipeline, test_pipeline_mesh): (
        Res<DisplayTextureHolder>, Res<DisplayTexturePipeline>, Res<DisplayTextureMesh>,
    )
) {
    // Render the texture
    let render_instance = render_instance.data.read().unwrap();
    let swapchain_frame = swapchain_frame.data.as_ref().unwrap();
    let mut command_buffer = WCommandBuffer::new(&render_instance, "display-texture");

    {
        let mut render_pass = command_buffer.create_render_pass("display-texture", |builder: &mut RenderPassBuilder| {
            builder.add_color_attachment(RenderPassColorAttachment {
                texture: Some(&swapchain_frame.view),
                ..Default::default()
            });
        });

        if display_texture_holders.texture.is_some() {
            if let (
                CachedPipelineStatus::OkRender(pipeline),
                Some(mesh),
                Some(bind_group)
            ) = (
                pipeline_manager.get_pipeline(texture_test_pipeline.index),
                meshes.get(&test_pipeline_mesh.mesh),
                &texture_test_pipeline.bind_group
            ) {
                // Set the pipeline
                if render_pass.set_pipeline(pipeline).is_ok() {
                    // Get the mesh
                    render_pass.set_vertex_buffer(0, &mesh.vertex_buffer);
                    render_pass.set_index_buffer(&mesh.index_buffer);

                    // Set bind group
                    render_pass.set_bind_group(0, bind_group);

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
    }

    // Submit the command buffer
    command_buffer.submit(&render_instance);
}
