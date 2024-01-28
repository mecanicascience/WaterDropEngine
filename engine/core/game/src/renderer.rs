use tracing::{debug, error};
use wde_ecs::{RenderComponentDynamic, RenderComponentStatic, TransformComponent, TransformUniform, World};
use wde_resources::{MaterialResource, ModelResource, ResourcesManager};
use wde_wgpu::{BindGroup, Buffer, BufferBindingType, BufferUsage, Color, CommandBuffer, LoadOp, Operations, RenderEvent, RenderInstance, RenderTexture, ShaderStages, StoreOp};

#[derive(Debug)]
pub struct Renderer {
    // Object matrices SSBO
    objects: Buffer,
    objects_bind_group: BindGroup,

    // Camera buffer bind group
    camera_buffer_bind_group: BindGroup,
}

impl Renderer {
    #[tracing::instrument]
    pub async fn new(render_instance: &RenderInstance, world: &mut World, res_manager: &mut ResourcesManager, camera_buffer: &mut Buffer) -> Self {
        // Maximum number of objects in a scene
        const MAX_OBJECTS: u32 = 10_000;
        
        // Create object matrices SSBO
        let mut objects = Buffer::new(
            &render_instance,
            "Object matrices SSBO",
            std::mem::size_of::<TransformUniform>() * MAX_OBJECTS as usize,
            BufferUsage::STORAGE | BufferUsage::MAP_WRITE,
            None);

        // Create object matrices SSBO bind group
        let objects_bind_group = objects.create_bind_group(
            &render_instance,
            BufferBindingType::Storage { read_only: true },
            ShaderStages::VERTEX).await;

        // Create camera buffer bind group
        let camera_buffer_bind_group = camera_buffer.create_bind_group(
            &render_instance,
            BufferBindingType::Uniform,
            ShaderStages::VERTEX).await;

        // Create instance
        Self {
            objects,
            objects_bind_group,
            camera_buffer_bind_group
        }
    }

    pub async fn update(&mut self, render_instance: &RenderInstance, world: &World, res_manager: &ResourcesManager, camera_buffer: &mut Buffer) {
        // Render entities
        for entity in world.entity_manager.living_entities.iter() {
            // Get render component dynamic
            if let Some(render_component) = world.get_component::<RenderComponentDynamic>(*entity) {
                // Create camera buffer bind group
                let camera_buffer_bind_group_layout = camera_buffer.create_bind_group_layout(
                    &render_instance,
                    BufferBindingType::Uniform,
                    ShaderStages::VERTEX).await;

                // Create object bind group layout
                let objects_bind_group_layout = self.objects.create_bind_group_layout(&render_instance,
                    BufferBindingType::Storage { read_only: true },
                    ShaderStages::VERTEX).await;

                // Get material
                if let Some(material) = res_manager.get_mut::<MaterialResource>(&render_component.material) {
                    // Check if pipeline is initialized
                    if !material.data.as_ref().unwrap().pipeline.is_initialized() {
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
    }

    #[tracing::instrument]
    pub fn update_ssbo(&self, render_instance: &RenderInstance, world: &World, update_static: bool) {
        // Update dynamic objects
        self.objects.map_mut(render_instance, |mut view| {
            // Cast data to TransformUniform
            let data = view.as_mut_ptr() as *mut TransformUniform;

            // Write data
            for entity in world.entity_manager.living_entities.iter() {
                if let Some(render_component) = world.get_component::<RenderComponentDynamic>(*entity) {
                    // Set data
                    let transform = world.get_component::<TransformComponent>(*entity).unwrap();
                    unsafe {
                        *data.add(render_component.id as usize) = TransformUniform::new(transform.clone());
                    }
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
            for entity in world.entity_manager.living_entities.iter() {
                if let Some(render_component) = world.get_component::<RenderComponentDynamic>(*entity) {
                    // Set data
                    let transform = world.get_component::<TransformComponent>(*entity).unwrap();
                    unsafe {
                        *data.add(render_component.id as usize) = TransformUniform::new(transform.clone());
                    }
                }
            }
        });
    }

    #[tracing::instrument]
    pub async fn render(&self, render_instance: &RenderInstance, world: &World, res_manager: &ResourcesManager) -> RenderEvent {
        // Handle render event
        let render_texture: RenderTexture = match RenderInstance::get_current_texture(&render_instance) {
            RenderEvent::Redraw(render_texture) => render_texture,
            event => return event,
        };

        // Render to texture
        debug!("Rendering to texture.");

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
                None);

            // Set bind groups
            render_pass.set_bind_group(0, &self.camera_buffer_bind_group);
            render_pass.set_bind_group(1, &self.objects_bind_group);

            // Render entities
            for entity in world.entity_manager.living_entities.iter() {
                // Get render component dynamic
                if let Some(render_component) = world.get_component::<RenderComponentDynamic>(*entity) {
                    // Get model
                    if let Some(model) = res_manager.get::<ModelResource>(&render_component.model) {
                        // Set model buffers
                        render_pass.set_vertex_buffer(0, &model.data.as_ref().unwrap().vertex_buffer);
                        render_pass.set_index_buffer(&model.data.as_ref().unwrap().index_buffer);

                        // Get material
                        if let Some(material) = res_manager.get_mut::<MaterialResource>(&render_component.material) {
                            // Check if pipeline is initialized
                            if !material.data.as_ref().unwrap().pipeline.is_initialized() {
                                continue;
                            }

                            // Set render pipeline
                            match render_pass.set_pipeline(&material.data.as_ref().unwrap().pipeline) {
                                Ok(_) => {
                                    let _ = render_pass.draw_indexed(0..model.data.as_ref().unwrap().index_count, render_component.id);
                                    continue;
                                },
                                Err(_) => {}
                            }
                        }
                    }
                }

                // Get render component static
                if let Some(render_component) = world.get_component::<RenderComponentStatic>(*entity) {
                    // Get model
                    if let Some(model) = res_manager.get::<ModelResource>(&render_component.model) {
                        // Set model buffers
                        render_pass.set_vertex_buffer(0, &model.data.as_ref().unwrap().vertex_buffer);
                        render_pass.set_index_buffer(&model.data.as_ref().unwrap().index_buffer);

                        // Get material
                        if let Some(material) = res_manager.get_mut::<MaterialResource>(&render_component.material) {
                            // Check if pipeline is initialized
                            if !material.data.as_ref().unwrap().pipeline.is_initialized() {
                                continue;
                            }

                            // Set render pipeline
                            match render_pass.set_pipeline(&material.data.as_ref().unwrap().pipeline) {
                                Ok(_) => {
                                    let _ = render_pass.draw_indexed(0..model.data.as_ref().unwrap().index_count, render_component.id);
                                    continue;
                                },
                                Err(_) => {}
                            }
                        }
                    }
                }
            }
        }

        // Submit command buffer
        command_buffer.submit(&render_instance);

        // Present frame
        let _ = render_instance.present(render_texture);

        // Return
        RenderEvent::None
    }
}
