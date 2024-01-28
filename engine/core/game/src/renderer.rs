use tracing::debug;
use wde_ecs::{RenderComponentDynamic, TransformComponent, TransformUniform, World};
use wde_resources::{ModelResource, ResourcesManager, ShaderResource};
use wde_wgpu::{Buffer, Color, CommandBuffer, LoadOp, Operations, RenderEvent, RenderInstance, RenderPipeline, ShaderStages, ShaderType, StoreOp};

#[derive(Debug)]
pub struct Renderer {
    // Object matrices SSBO
    objects: Buffer,
    objects_bind_group: wde_wgpu::BindGroup,

    // Camera buffer bind group
    camera_buffer_bind_group: wde_wgpu::BindGroup,

    // Render pipeline
    render_pipeline: wde_wgpu::RenderPipeline,
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
            wde_wgpu::BufferUsage::STORAGE | wde_wgpu::BufferUsage::MAP_WRITE,
            None);

        // Create object matrices SSBO bind group and bind group layout
        let objects_bind_group_layout = objects.create_bind_group_layout(
            &render_instance,
            wde_wgpu::BufferBindingType::Storage { read_only: true },
            ShaderStages::VERTEX).await;
        let objects_bind_group = objects.create_bind_group(
            &render_instance,
            wde_wgpu::BufferBindingType::Storage { read_only: true },
            ShaderStages::VERTEX).await;

        // Create camera buffer bind group
        let camera_buffer_bind_group = camera_buffer.create_bind_group(
            &render_instance,
            wde_wgpu::BufferBindingType::Uniform,
            ShaderStages::VERTEX).await;



        // Create shaders
        let vertex_shader_handle = res_manager.load::<ShaderResource>("shaders/unicolor/vert");
        let fragment_shader_handle = res_manager.load::<ShaderResource>("shaders/unicolor/frag");

        // Wait for shaders to load
        res_manager.wait_for(&vertex_shader_handle, &render_instance).await;
        res_manager.wait_for(&fragment_shader_handle, &render_instance).await;

        // Create camera bind group layout
        let camera_buffer_bind_group_layout = camera_buffer.create_bind_group_layout(
            &render_instance,
            wde_wgpu::BufferBindingType::Uniform,
            ShaderStages::VERTEX).await;

        // Create default render pipeline
        let mut render_pipeline = RenderPipeline::new("Main Render");
        let _ = render_pipeline
            .set_shader(&res_manager.get::<ShaderResource>(&vertex_shader_handle).unwrap().data.as_ref().unwrap().module, ShaderType::Vertex)
            .set_shader(&res_manager.get::<ShaderResource>(&fragment_shader_handle).unwrap().data.as_ref().unwrap().module, ShaderType::Fragment)
            .add_bind_group(camera_buffer_bind_group_layout)
            .add_bind_group(objects_bind_group_layout)
            .init(&render_instance).await;

        // Create instance
        Self {
            objects,
            objects_bind_group,
            camera_buffer_bind_group,
            render_pipeline,
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
    pub async fn render(renderer: &Renderer, render_instance: &RenderInstance, world: &World, res_manager: &ResourcesManager) -> RenderEvent {
        // Handle render event
        let render_texture: wde_wgpu::RenderTexture = match RenderInstance::get_current_texture(&render_instance) {
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
            render_pass.set_bind_group(0, &renderer.camera_buffer_bind_group);
            render_pass.set_bind_group(1, &renderer.objects_bind_group);

            // Render entities
            for entity in world.entity_manager.living_entities.iter() {
                // Get render component dynamic
                if let Some(render_component) = world.get_component::<RenderComponentDynamic>(*entity) {
                    // Get model
                    if let Some(model) = res_manager.get::<ModelResource>(&render_component.model) {
                        // Set model buffers
                        render_pass.set_vertex_buffer(0, &model.data.as_ref().unwrap().vertex_buffer);
                        render_pass.set_index_buffer(&model.data.as_ref().unwrap().index_buffer);

                        // Set render pipeline
                        match render_pass.set_pipeline(&renderer.render_pipeline) {
                            Ok(_) => {
                                let _ = render_pass.draw_indexed(0..model.data.as_ref().unwrap().index_count, render_component.id);
                                continue;
                            },
                            Err(_) => {}
                        }
                    }
                }

                // Get render component static
                if let Some(render_component) = world.get_component::<RenderComponentDynamic>(*entity) {
                    // Get model
                    if let Some(model) = res_manager.get::<ModelResource>(&render_component.model) {
                        // Set model buffers
                        render_pass.set_vertex_buffer(0, &model.data.as_ref().unwrap().vertex_buffer);
                        render_pass.set_index_buffer(&model.data.as_ref().unwrap().index_buffer);

                        // Set render pipeline
                        match render_pass.set_pipeline(&renderer.render_pipeline) {
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

        // Submit command buffer
        command_buffer.submit(&render_instance);

        // Present frame
        let _ = render_instance.present(render_texture);

        // Return
        RenderEvent::None
    }
}
