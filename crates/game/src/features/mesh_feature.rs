use bevy::{prelude::*, window::WindowResized};
use wde_render::{assets::{Buffer, GpuBuffer, GpuMesh, GpuTexture, Mesh, RenderAssets, Texture, TextureUsages}, components::TransformUniform, core::{extract_macros::ExtractWorld, Extract, Render, RenderApp, RenderSet, SwapchainFrame}, features::{CameraFeatureBuffer, CameraFeatureLayout}, pipelines::{CachedPipelineIndex, CachedPipelineStatus, PipelineManager, RenderPipelineDescriptor}};
use wde_wgpu::{bind_group::{BindGroup, BindGroupLayout, WgpuBindGroupLayout}, buffer::{BufferBindingType, BufferUsage}, command_buffer::{Color, LoadOp, Operations, StoreOp, WCommandBuffer}, instance::WRenderInstance, render_pipeline::WShaderStages, texture::WTexture};

/// The maximum number of batches to render using the mesh feature.
pub const MAX_BATCHES_COUNT: usize = 100;

pub struct MeshFeature;
impl Plugin for MeshFeature {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, init_depth)
            .add_systems(Update, resize_depth);


        app.get_sub_app_mut(RenderApp).unwrap()
            .add_systems(Extract, (construct_pass, extract_depth_texture))
            .add_systems(Render, render.in_set(RenderSet::Render))
            .init_resource::<MeshPipeline>();
    }

    fn finish(&self, app: &mut App) {
        // Create the ssbo
        let buffer: Handle<Buffer> = app.world_mut().add_asset(Buffer {
            label: "mesh_ssbo_cpu".to_string(),
            size: std::mem::size_of::<TransformUniform>() * MAX_BATCHES_COUNT,
            usage: BufferUsage::COPY_SRC | BufferUsage::MAP_WRITE,
            content: None,
        });
        let buffer_gpu: Handle<Buffer> = app.world_mut().add_asset(Buffer {
            label: "mesh_ssbo_gpu".to_string(),
            size: std::mem::size_of::<TransformUniform>() * MAX_BATCHES_COUNT,
            usage: BufferUsage::STORAGE | BufferUsage::COPY_DST,
            content: None,
        });

        // Add resources
        app.get_sub_app_mut(RenderApp).unwrap()
            .insert_resource(RenderMeshPass {
                ssbo: buffer,
                ssbo_gpu: buffer_gpu,
                batches: Vec::new()
            });
    }
}


#[derive(Resource)]
pub struct MeshPipeline {
    pub index: CachedPipelineIndex,
    pub ssbo_layout_built: WgpuBindGroupLayout,
}
impl FromWorld for MeshPipeline {
    fn from_world(world: &mut World) -> Self {
        // Get the camera layout
        let camera_layout = &world.get_resource::<CameraFeatureLayout>().unwrap().layout;

        // Create the ssbo layout
        let render_instance = world.get_resource::<WRenderInstance<'static>>().unwrap();
        let ssbo_layout = BindGroupLayout::new("mesh_ssbo", |builder| {
            builder.add_buffer(0,
                WShaderStages::VERTEX,
                BufferBindingType::Storage { read_only: true });
        });
        let ssbo_layout_built = ssbo_layout.build(&render_instance.data.read().unwrap());

        // Create the pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: "mesh",
            vert: Some(world.load_asset("mesh/vert.wgsl")),
            frag: Some(world.load_asset("mesh/frag.wgsl")),
            bind_group_layouts: vec![camera_layout.clone(), ssbo_layout.clone()],
            depth_stencil: true,
            ..Default::default()
        };
        let cached_index = world.get_resource_mut::<PipelineManager>().unwrap().create_render_pipeline(pipeline_desc);
        
        MeshPipeline { index: cached_index, ssbo_layout_built }
    }
}




#[derive(Resource)]
pub struct MeshDepthTexture {
    pub texture: Handle<Texture>
}

fn init_depth(mut commands: Commands, server: Res<AssetServer>, window: Query<&Window>) {
    let resolution = &window.single().resolution;
    let texture = server.add(Texture {
        label: "depth".to_string(),
        size: (resolution.width() as usize, resolution.height() as usize, 1),
        format: WTexture::DEPTH_FORMAT,
        usages: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        ..Default::default()
    });
    commands.insert_resource(MeshDepthTexture { texture });
}

fn resize_depth(mut commands: Commands, mut window_resized_events: EventReader<WindowResized>, server: Res<AssetServer>) {
    for event in window_resized_events.read() {
        // Recreate the depth texture
        let texture = server.add(Texture {
            label: "depth".to_string(),
            size: (event.width as usize, event.height as usize, 1),
            format: WTexture::DEPTH_FORMAT,
            usages: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            ..Default::default()
        });
        commands.insert_resource(MeshDepthTexture { texture });
    }
}

fn extract_depth_texture(mut commands: Commands, depth_texture : ExtractWorld<Res<MeshDepthTexture>>) {
    commands.insert_resource(MeshDepthTexture { texture: depth_texture.texture.clone() });
}




pub struct RenderMeshBatch {
    pub mesh: Handle<Mesh>,
    pub first: usize,
    pub count: usize,
}

#[derive(Resource)]
pub struct RenderMeshPass {
    pub ssbo: Handle<Buffer>,
    pub ssbo_gpu: Handle<Buffer>,
    pub batches: Vec<RenderMeshBatch>,
}

fn construct_pass(
    mut pass: ResMut<RenderMeshPass>, render_instance: Res<WRenderInstance<'static>>,
    entities: ExtractWorld<Query<(&Transform, &Handle<Mesh>)>>, meshes: Res<RenderAssets<GpuMesh>>,
    buffers: Res<RenderAssets<GpuBuffer>>
) {
    // Clear the batches of the previous frame
    pass.batches.clear();

    // Get the ssbo
    let ssbo = match buffers.get(&pass.ssbo) {
        Some(ssbo) => ssbo,
        None => return
    };
    
    // If no entities, return
    if entities.is_empty() {
        return
    }

    // Create the batches
    let render_instance = render_instance.data.read().unwrap();
    ssbo.buffer.map_write(&render_instance, |mut view| {
        let mut first = 0;
        let mut count = 1;
        let mut last_mesh: Option<Handle<Mesh>> = None;
        let data = view.as_mut_ptr() as *mut TransformUniform;

        for (transform, mesh_handle) in entities.iter() {
            // Check if new element in same batch
            let last_mesh_ref = last_mesh.as_ref();
            if last_mesh_ref.is_some() {
                if mesh_handle.id() == last_mesh_ref.unwrap().id() {
                    // Update the ssbo
                    let transform = TransformUniform::new(transform);
                    unsafe {
                        *data.add(first + count) = transform;
                    }

                    // Increment the count
                    count += 1;

                    continue;
                } else {
                    // Push the batch
                    pass.batches.push(RenderMeshBatch {
                        mesh: last_mesh_ref.unwrap().clone_weak(),
                        first,
                        count
                    });

                    // Reset the batch
                    first += count;
                    count = 1;
                    last_mesh = None;
                }
            }

            // Update the last mesh and ssbo if loaded
            if meshes.get(mesh_handle).is_some() {
                // Update the mesh
                last_mesh = Some(mesh_handle.clone_weak());

                // Update the ssbo
                let transform = TransformUniform::new(transform);
                unsafe {
                    *data.add(first) = transform;
                }
            }
        }

        // Push the last batch
        if let Some(last_mesh) = last_mesh {
            pass.batches.push(RenderMeshBatch {
                mesh: last_mesh,
                first,
                count
            });
        }
    });

    // Update the ssbo
    let ssbo_gpu = match buffers.get(&pass.ssbo_gpu) {
        Some(ssbo) => ssbo,
        None => return
    };
    ssbo_gpu.buffer.copy_from_buffer(&render_instance, &ssbo.buffer);
}

fn render(
    (render_instance, swapchain_frame, pipeline_manager): (
        Res<WRenderInstance<'static>>, Res<SwapchainFrame>,  Res<PipelineManager>
    ),
    (camera_buffer, camera_layout) : (
        Res<CameraFeatureBuffer>, Res<CameraFeatureLayout>
    ),
    (meshes, buffers, textures): (
        Res<RenderAssets<GpuMesh>>, Res<RenderAssets<GpuBuffer>>, Res<RenderAssets<GpuTexture>>
    ),
    (mesh_pipeline, render_mesh_pass, depth_texture): (
        Res<MeshPipeline>, Res<RenderMeshPass>, Res<MeshDepthTexture>
    )
) {
    // Get the render instance and swapchain frame
    let render_instance = render_instance.data.read().unwrap();
    let swapchain_frame = swapchain_frame.data.as_ref().unwrap();

    // Check if depth texture is ready
    let depth_texture = match textures.get(&depth_texture.texture) {
        Some(tex) => if render_instance.surface_config.as_ref().unwrap().width == tex.texture.size.0
            && render_instance.surface_config.as_ref().unwrap().height == tex.texture.size.1 {
            tex
        } else {
            return
        },
        None => return
    };

    // Create the render pass
    let mut command_buffer = WCommandBuffer::new(&render_instance, "mesh");
    {
        let mut render_pass = command_buffer.create_render_pass(
            "mesh", &swapchain_frame.view,
            Some(Operations {
                load: LoadOp::Clear(Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
                store: StoreOp::Store,
            }),
            Some(&depth_texture.texture.view),
        );

        // Render the mesh
        if let (
            CachedPipelineStatus::Ok(pipeline),
            Some(camera_buffer),
            Some(ssbo_buffer)
        ) = (
            pipeline_manager.get_pipeline(mesh_pipeline.index),
            buffers.get(&camera_buffer.buffer),
            buffers.get(&render_mesh_pass.ssbo_gpu)
        ) {
            // Set the camera bind group
            let bind_group = BindGroup::build("camera", &render_instance, &camera_layout.layout_built, &vec![
                BindGroup::buffer(0, &camera_buffer.buffer)
            ]);
            render_pass.set_bind_group(0, &bind_group);

            // Set the pipeline
            if render_pass.set_pipeline(pipeline).is_ok() {
                // Set the ssbo
                let bind_group = BindGroup::build("mesh_ssbo", &render_instance, &mesh_pipeline.ssbo_layout_built, &vec![
                    BindGroup::buffer(0, &ssbo_buffer.buffer)
                ]);
                render_pass.set_bind_group(1, &bind_group);

                for batch in render_mesh_pass.batches.iter() {
                    // Get the mesh
                    let mesh = match meshes.get(&batch.mesh) {
                        Some(mesh) => mesh,
                        None => continue
                    };

                    // Set the mesh buffers
                    render_pass.set_vertex_buffer(0, &mesh.vertex_buffer);
                    render_pass.set_index_buffer(&mesh.index_buffer);

                    // Draw the mesh
                    let instance_indices = batch.first as u32..((batch.first + batch.count) as u32);
                    match render_pass.draw_indexed(0..mesh.index_count, instance_indices) {
                        Ok(_) => {},
                        Err(e) => {
                            error!("Failed to draw: {:?}.", e);
                        }
                    };
                }
            } else {
                error!("Failed to set pipeline.");
            }
        }
    }

    // Submit the command buffer
    command_buffer.submit(&render_instance);
}
