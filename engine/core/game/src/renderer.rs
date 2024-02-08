use std::ops::Range;

use tracing::{debug, error, trace, warn};
use wde_ecs::{CameraComponent, CameraUniform, EntityIndex, RenderComponent, RenderComponentInstanced, RenderComponentSSBODynamic, RenderComponentSSBOStatic, TransformComponent, TransformUniform, World, MAX_ENTITIES};
use wde_resources::{MaterialResource, ModelResource, Resource, ResourceHandle, ResourcesManager};
use wde_wgpu::{BindGroup, Buffer, BufferBindingType, BufferUsage, Color, CommandBuffer, LoadOp, Operations, RenderEvent, RenderInstance, RenderTexture, ShaderStages, StoreOp, Texture, TextureUsages};

/// Describes a draw batch.
#[derive(Debug)]
struct IndirectBatch {
    /// Model of the set of entities
    model: ResourceHandle,
    /// Material of the set of entities
    material: ResourceHandle,
    /// Entity list (Note that the index need to be the same as the index in the SSBO)
    entities: Range<u32>,
}


#[derive(Debug)]
pub struct Renderer {
    // Object matrices SSBO
    objects: Buffer,
    objects_bind_group: BindGroup,

    // Camera buffer
    camera_buffer: Buffer,
    camera_buffer_bind_group: BindGroup,

    // Depth texture
    depth_texture: Texture,
}

impl Renderer {
    /// Create a new renderer instance.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    /// * `world` - The world of the scene
    /// * `res_manager` - The resources manager
    /// * `camera_buffer` - The camera buffer
    #[tracing::instrument]
    pub async fn new(render_instance: &RenderInstance<'_>, world: &mut World, res_manager: &mut ResourcesManager) -> Self {
        // Create object matrices SSBO
        let mut objects = Buffer::new(
            &render_instance,
            "Object matrices SSBO",
            std::mem::size_of::<TransformUniform>() * MAX_ENTITIES as usize,
            BufferUsage::STORAGE | BufferUsage::MAP_WRITE,
            None);

        // Create object matrices SSBO bind group
        let objects_bind_group = objects.create_bind_group(
            &render_instance,
            BufferBindingType::Storage { read_only: true },
            ShaderStages::VERTEX).await;


        // Create camera uniform buffer
        let mut camera_buffer = Buffer::new(
            &render_instance,
            "Camera buffer",
            std::mem::size_of::<CameraUniform>(),
            wde_wgpu::BufferUsage::UNIFORM | wde_wgpu::BufferUsage::COPY_DST,
            None);

        // Create camera buffer bind group
        let camera_buffer_bind_group = camera_buffer.create_bind_group(
            &render_instance,
            BufferBindingType::Uniform,
            ShaderStages::VERTEX).await;


        // Create depth texture
        let depth_texture = Texture::new(
            render_instance,
            "Depth texture",
            (render_instance.surface_config.as_ref().unwrap().width, render_instance.surface_config.as_ref().unwrap().height),
            Texture::DEPTH_FORMAT,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING).await;

        // Create instance
        Self {
            objects,
            objects_bind_group,
            camera_buffer,
            camera_buffer_bind_group,
            depth_texture,
        }
    }
    


    /// Update the renderer instance.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    /// * `world` - The world of the scene
    /// * `res_manager` - The resources manager
    /// * `camera_buffer` - The camera buffer
    #[tracing::instrument]
    pub async fn init_pipelines(&mut self, render_instance: &RenderInstance<'_>, world: &World, res_manager: &ResourcesManager) {
        // Initialize pipelines
        for entity in world.get_entities_with_component::<RenderComponent>().iter() {
            // Get render component
            if let Some(render_component) = world.get_component::<RenderComponent>(*entity) {
                // Check if pipeline is initialized
                if let Some(material) = res_manager.get_mut::<MaterialResource>(&render_component.material) {
                    if material.data.as_ref().unwrap().pipeline.is_initialized() {
                        continue;
                    }
                }

                // Create camera buffer bind group
                let camera_buffer_bind_group_layout = self.camera_buffer.create_bind_group_layout(
                    &render_instance,
                    BufferBindingType::Uniform,
                    ShaderStages::VERTEX).await;

                // Create object bind group layout
                let objects_bind_group_layout = self.objects.create_bind_group_layout(&render_instance,
                    BufferBindingType::Storage { read_only: true },
                    ShaderStages::VERTEX).await;

                // Initialize pipeline
                if let Some(material) = res_manager.get_mut::<MaterialResource>(&render_component.material) {
                    material.data.as_mut().unwrap().pipeline
                        .add_bind_group(camera_buffer_bind_group_layout)
                        .add_bind_group(objects_bind_group_layout)
                        .init(&render_instance).await
                        .unwrap_or_else(|_| {
                            error!("Failed to initialize pipeline for material {}.", material.label);
                        });
                }
            }
        }

        // Initialize pipelines instanced
        for entity in world.get_entities_with_component::<RenderComponentInstanced>().iter() {
            // Get render component instanced
            if let Some(render_component) = world.get_component::<RenderComponentInstanced>(*entity) {
                // Check if pipeline is initialized
                if let Some(material) = res_manager.get_mut::<MaterialResource>(&render_component.material) {
                    if material.data.as_ref().unwrap().pipeline.is_initialized() {
                        continue;
                    }
                }

                // Create camera buffer bind group
                let camera_buffer_bind_group_layout = self.camera_buffer.create_bind_group_layout(
                    &render_instance,
                    BufferBindingType::Uniform,
                    ShaderStages::VERTEX).await;

                // Create object bind group layout
                let objects_bind_group_layout = self.objects.create_bind_group_layout(&render_instance,
                    BufferBindingType::Storage { read_only: true },
                    ShaderStages::VERTEX).await;

                // Initialize pipeline
                if let Some(material) = res_manager.get_mut::<MaterialResource>(&render_component.material) {
                    material.data.as_mut().unwrap().pipeline
                        .add_bind_group(camera_buffer_bind_group_layout)
                        .add_bind_group(objects_bind_group_layout)
                        .init(&render_instance).await
                        .unwrap_or_else(|_| {
                            error!("Failed to initialize pipeline for material {}.", material.label);
                        });
                }
            }
        }
    }



    /// Update the renderer instance SSBO.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    /// * `world` - The world
    /// * `update_static` - True if static objects should be updated
    #[tracing::instrument]
    pub fn update_ssbo(&self, render_instance: &RenderInstance, world: &World, update_static: bool) {
        // Update dynamic objects
        self.objects.map_mut(render_instance, |mut view| {
            // Cast data to TransformUniform
            let data = view.as_mut_ptr() as *mut TransformUniform;

            // Write data
            for entity in world.get_entities_with_component::<RenderComponentSSBODynamic>().iter() {
                // Get render component dynamic
                if let Some(render_component) = world.get_component::<RenderComponentSSBODynamic>(*entity) {
                    // Set data
                    let transform = world.get_component::<TransformComponent>(*entity).unwrap();
                    unsafe {
                        *data.add(render_component.id as usize) = TransformUniform::new(transform);
                    };
                }
            }
        });

        // Update static objects
        if !update_static {
            return;
        }
        self.objects.map_mut(render_instance, |mut view| {
            // Cast data to TransformUniform
            let data = view.as_mut_ptr() as *mut TransformUniform;

            // Write data
            for entity in world.get_entities_with_component::<RenderComponentSSBOStatic>().iter() {
                // Get render component static
                if let Some(render_component) = world.get_component::<RenderComponentSSBOStatic>(*entity) {
                    // Set data
                    let transform = world.get_component::<TransformComponent>(*entity).unwrap();
                    unsafe {
                        *data.add(render_component.id as usize) = TransformUniform::new(transform);
                    };
                }
            }
        });
    }


    /// Render the renderer instance.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    /// * `world` - The world
    /// * `res_manager` - The resources manager
    #[tracing::instrument]
    pub async fn render(&self, render_instance: &RenderInstance<'_>, world: &World, res_manager: &ResourcesManager) -> RenderEvent {
        debug!("Starting render.");

        // Handle render event
        let render_texture: RenderTexture = match RenderInstance::get_current_texture(&render_instance) {
            RenderEvent::Redraw(render_texture) => render_texture,
            event => return event,
        };

        // Create draw batches
        let mut draws_batches: Vec<IndirectBatch> = Vec::new();
        {
            trace!("Creating draw batches.");
            let _trace_draws = tracing::span!(tracing::Level::TRACE, "create_batches");

            let first_entity = match world.get_entities_with_component::<RenderComponent>().iter().next() {
                Some(entity) => {
                    world.get_component::<RenderComponent>(*entity).unwrap()
                }
                None => return RenderEvent::None,
            };
            draws_batches.push(IndirectBatch {
                model: first_entity.model.clone(),
                material: first_entity.material.clone(),
                entities: first_entity.id..(first_entity.id + 1),
            });

            // Create draw batches for entities with render component
            let mut first_batch = true;
            for entity in world.get_entities_with_component::<RenderComponent>().iter() {
                // Ignore first entity
                if first_batch {
                    first_batch = false;
                    continue;
                }

                // Compare model and material with the last draw
                let entity_render = world.get_component::<RenderComponent>(*entity).unwrap();
                let same_model = draws_batches.last().unwrap().model.index == entity_render.model.index;
                let same_material = draws_batches.last().unwrap().material.index == entity_render.material.index;
                let contiguous = entity_render.id == draws_batches.last().unwrap().entities.end;

                // Handle draw creation
                if same_model && same_material && contiguous {
                    // Add entity to the last draw
                    let last_draw = draws_batches.last_mut().unwrap();
                    last_draw.entities.end += 1;
                }
                else {
                    // Create a new draw
                    let new_draw = IndirectBatch {
                        model: entity_render.model.clone(),
                        material: entity_render.material.clone(),
                        entities: entity_render.id..(entity_render.id + 1),
                    };
                    draws_batches.push(new_draw);
                }
            }

            // Create draw batches for entities with render component instanced
            for entity in world.get_entities_with_component::<RenderComponentInstanced>().iter() {
                // Compare model and material with the last draw
                let entity_render = world.get_component::<RenderComponentInstanced>(*entity).unwrap();
                let same_model = draws_batches.last().unwrap().model.index == entity_render.model.index;
                let same_material = draws_batches.last().unwrap().material.index == entity_render.material.index;

                // Handle draw creation
                if same_model && same_material && entity_render.ids.start == draws_batches.last().unwrap().entities.end {
                    // Add entity to the last draw
                    let last_draw = draws_batches.last_mut().unwrap();
                    last_draw.entities.end += entity_render.ids.end - entity_render.ids.start;
                }
                else {
                    // Create a new draw
                    let new_draw = IndirectBatch {
                        model: entity_render.model.clone(),
                        material: entity_render.material.clone(),
                        entities: entity_render.ids.clone(),
                    };
                    draws_batches.push(new_draw);
                }
            }
        }

        // // Create draw indirect commands
        // let mut draw_indirect_commands: Vec<DrawIndexedIndirect> = Vec::new();
        // {
        //     trace!("Creating draw indirect commands.");
        //     let _trace_draws = tracing::span!(tracing::Level::TRACE, "create_draw_indirect");

        //     for draw in draws_batches.iter() {
        //         // Get model
        //         if let Some(model) = res_manager.get::<ModelResource>(&draw.model) {
        //             // Check if model is initialized
        //             if !model.loaded() {
        //                 continue;
        //             }

        //             // Create draw indirect command
        //             let draw_indirect = DrawIndexedIndirect {
        //                 vertex_count: model.data.as_ref().unwrap().vertex_count,
        //                 instance_count: draw.entities.end - draw.entities.start,
        //                 base_index: 0,
        //                 vertex_offset: 0,
        //                 base_instance: draw.entities.start,
        //             };
        //             draw_indirect_commands.push(draw_indirect);
        //         }
        //     }
        // }

        // Render
        {
            trace!("Rendering batches.");
            let _trace_render = tracing::span!(tracing::Level::TRACE, "render_batches");

            // Create command buffer
            let mut command_buffer = CommandBuffer::new(
                    &render_instance, "Main render").await;

            {
                // Create render pass
                let mut render_pass = command_buffer.create_render_pass(
                    "Main render",
                    &render_texture.view,
                    Some(Operations {
                        load: LoadOp::Clear(Color { r : 0.1, g: 0.105, b: 0.11, a: 1.0 }),
                        store: StoreOp::Store,
                    }),
                    Some(&self.depth_texture.view));

                // Set bind groups
                render_pass.set_bind_group(0, &self.camera_buffer_bind_group);
                render_pass.set_bind_group(1, &self.objects_bind_group);
                
                // Last model and material
                let mut last_model: Option<usize> = None;
                let mut last_material: Option<usize> = None;

                // Render entities
                for draw in draws_batches.iter() {
                    // Get model
                    if let Some(model) = res_manager.get::<ModelResource>(&draw.model) {
                        // Check if model is initialized
                        if !model.loaded() {
                            continue;
                        }

                        // Set model buffers
                        if last_model.is_none() || last_model.unwrap() != draw.model.index {
                            render_pass.set_vertex_buffer(0, &model.data.as_ref().unwrap().vertex_buffer);
                            render_pass.set_index_buffer(&model.data.as_ref().unwrap().index_buffer);
                            last_model = Some(draw.model.index);
                        }

                        // Get material
                        if let Some(material) = res_manager.get_mut::<MaterialResource>(&draw.material) {
                            // Check if pipeline is initialized
                            if !material.data.as_ref().unwrap().pipeline.is_initialized() {
                                continue;
                            }

                            // Set render pipeline
                            if last_material.is_none() || last_material.unwrap() != draw.material.index {
                                if render_pass.set_pipeline(&material.data.as_ref().unwrap().pipeline).is_err() {
                                    continue;
                                }
                                last_material = Some(draw.material.index);
                            }

                            // Draw
                            render_pass
                                .draw_indexed(0..model.data.as_ref().unwrap().index_count, draw.entities.clone())
                                .unwrap_or_else(|_| {
                                    error!("Failed to draw batch {:?}.", draw);
                                });
                        }
                    }
                }
            }

            // Submit command buffer
            command_buffer.submit(&render_instance);
        }

        // Present frame
        let _ = render_instance.present(render_texture);

        // Return
        RenderEvent::None
    }



    /// Update the active camera.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    /// * `camera_buffer` - The camera buffer
    #[tracing::instrument]
    pub fn update_camera(&mut self, render_instance: &RenderInstance, world: &World, camera: EntityIndex) {
        // Create camera uniform
        let mut camera_uniform = CameraUniform::new();
        let surface_config = render_instance.surface_config.as_ref().unwrap();
        camera_uniform.world_to_screen = CameraUniform::get_world_to_screen(
            CameraComponent {
                aspect: surface_config.width as f32 / surface_config.height as f32,
                fovy: 60.0, znear: 0.1, zfar: 1000.0
            },
            world.get_component::<TransformComponent>(camera).unwrap().clone()
        ).into();

        // Write camera buffer
        self.camera_buffer.write(&render_instance, bytemuck::cast_slice(&[camera_uniform]), 0);
    }



    #[tracing::instrument]
    pub async fn resize(&mut self, render_instance: &RenderInstance<'_>, width: u32, height: u32) {
        // Recreate depth texture
        self.depth_texture = Texture::new(
            render_instance,
            "Depth texture",
            (width, height),
            Texture::DEPTH_FORMAT,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING).await;
    }
}
