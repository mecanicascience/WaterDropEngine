use tracing::{debug, error, trace, warn};
use wde_ecs::{CameraComponent, CameraUniform, EntityIndex, RenderComponent, RenderComponentInstanced, RenderComponentSSBODynamic, RenderComponentSSBOStatic, TransformComponent, TransformUniform, World, MAX_ENTITIES};
use wde_resources::{MaterialResource, ModelResource, Resource, ResourceHandle, ResourcesManager, ShaderResource};
use wde_wgpu::{BindGroup, Buffer, BufferBindingType, BufferUsage, Color, CommandBuffer, ComputePipeline, DrawIndexedIndirectArgs, LoadOp, Operations, RenderEvent, RenderInstance, RenderTexture, ShaderStages, StoreOp, Texture, TextureUsages};

/// Describes the maximum number of indirect commands.
const MAX_INDIRECT_COMMANDS: usize = 1_000_000;

/// Describes a draw batch.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct IndirectBatch {
    /// First entity index (Note that the index need to be the same as the index in the SSBO)
    first: u32,
    /// Number of entities
    count: u32,
    /// Number of indices in the model
    index_count: u32,
    /// Batch index. This uniquely identifies a model and material pair.
    batch_index: u32,
}

/// Describes a set of draw indirect commands.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct DrawIndexedIndirectDesc {
    /// First draw indirect command index
    first: u32,
    /// Number of draw indirect commands
    count: u32,
    /// Batch index. This uniquely identifies a model and material pair.
    batch_index: u32,
    /// Padding
    _padding: u32,
}

/// Describes the draw indirect data.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct DrawIndirectData {
    /// The number of descriptors that will generate indirect commands
    descriptor_count: u32,
}


#[derive(Debug)]
pub struct Renderer {
    // Object matrices SSBO
    objects: Buffer,
    objects_bind_group: BindGroup,

    // Render buffers
    batch_commands_buffer: Buffer,
    batch_commands_buffer_bind_group: BindGroup,
    indirect_commands_buffer: Buffer,
    indirect_commands_buffer_bind_group: BindGroup,
    draw_indirect_desc_buffer: Buffer,
    draw_indirect_desc_buffer_bind_group: BindGroup,
    _draw_indirect_desc_buffer_temporary: Buffer,
    draw_indirect_desc_buffer_temporary_bind_group_write: BindGroup,
    draw_indirect_desc_buffer_temporary_bind_group_read: BindGroup,
    draw_indirect_data_buffer: Buffer,
    draw_indirect_data_buffer_bind_group: BindGroup,
    record_indirect_compute_pipeline: ComputePipeline,
    record_indirect_compute_instructions_pipeline: ComputePipeline,
    
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
        // ==== Object matrices SSBO ====
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


        // ==== Indirect draw commands ====
        // Create batch indirect commands buffer
        let mut batch_commands_buffer = Buffer::new(
            &render_instance,
            "Batch commands buffer",
            std::mem::size_of::<IndirectBatch>() * MAX_INDIRECT_COMMANDS as usize,
            BufferUsage::MAP_WRITE | BufferUsage::STORAGE,
            None);
        let batch_commands_buffer_bind_group = batch_commands_buffer.create_bind_group(
            &render_instance,
            BufferBindingType::Storage { read_only: true },
            ShaderStages::COMPUTE).await;

        // Create draw indirect descriptor buffer temporary
        let mut draw_indirect_desc_buffer_temporary = Buffer::new(
            &render_instance,
            "Draw indirect descriptor buffer temporary",
            std::mem::size_of::<DrawIndexedIndirectDesc>() * MAX_INDIRECT_COMMANDS as usize,
            BufferUsage::STORAGE,
            None);
        let draw_indirect_desc_buffer_temporary_bind_group_write = draw_indirect_desc_buffer_temporary.create_bind_group(
            &render_instance,
            BufferBindingType::Storage { read_only: false },
            ShaderStages::COMPUTE).await;
        let draw_indirect_desc_buffer_temporary_bind_group_read = draw_indirect_desc_buffer_temporary.create_bind_group(
            &render_instance,
            BufferBindingType::Storage { read_only: true },
            ShaderStages::COMPUTE).await;

        // Create draw indirect descriptor buffer
        let mut draw_indirect_desc_buffer = Buffer::new(
            &render_instance,
            "Draw indirect descriptor buffer",
            std::mem::size_of::<DrawIndexedIndirectDesc>() * MAX_INDIRECT_COMMANDS as usize,
            BufferUsage::STORAGE | BufferUsage::MAP_READ,
            None);
        let draw_indirect_desc_buffer_bind_group = draw_indirect_desc_buffer.create_bind_group(
            &render_instance,
            BufferBindingType::Storage { read_only: false },
            ShaderStages::COMPUTE).await;

        // Create draw indirect data buffer
        let mut draw_indirect_data_buffer = Buffer::new(
            &render_instance,
            "Draw indirect data buffer",
            std::mem::size_of::<DrawIndirectData>() as usize,
            BufferUsage::STORAGE | BufferUsage::MAP_READ | BufferUsage::MAP_WRITE,
            None);
        let draw_indirect_data_buffer_bind_group = draw_indirect_data_buffer.create_bind_group(
            &render_instance,
            BufferBindingType::Storage { read_only: false },
            ShaderStages::COMPUTE).await;

        // Create draw indirect commands buffer
        let mut indirect_commands_buffer = Buffer::new(
            &render_instance,
            "Draw indirect commands buffer",
            std::mem::size_of::<DrawIndexedIndirectArgs>() * MAX_INDIRECT_COMMANDS as usize,
            BufferUsage::INDIRECT | BufferUsage::STORAGE,
            None);
        let indirect_commands_buffer_bind_group = indirect_commands_buffer.create_bind_group(
            &render_instance,
            BufferBindingType::Storage { read_only: false },
            ShaderStages::COMPUTE).await;

        // Create compute pipeline for indirect draw commands
        let compute_shader = res_manager.load::<ShaderResource>("compute/record_draw_commands");
        res_manager.wait_for(&compute_shader, render_instance).await;
        let mut record_indirect_compute_pipeline = ComputePipeline::new("Draw indirect");
        if record_indirect_compute_pipeline
            .set_shader(&res_manager.get::<ShaderResource>(&compute_shader).unwrap().data.as_ref().unwrap().module)
            .add_bind_group(batch_commands_buffer.create_bind_group_layout(
                render_instance, BufferBindingType::Storage { read_only: true }, ShaderStages::COMPUTE).await)
            .add_bind_group(draw_indirect_desc_buffer_temporary.create_bind_group_layout(
                render_instance, BufferBindingType::Storage { read_only: false }, ShaderStages::COMPUTE).await)
            .add_bind_group(indirect_commands_buffer.create_bind_group_layout(
                render_instance, BufferBindingType::Storage { read_only: false }, ShaderStages::COMPUTE).await)
            .init(&render_instance).is_err() {
            error!("Failed to initialize compute pipeline.");
        }

        // Create compute pipeline
        let compute_shader = res_manager.load::<ShaderResource>("compute/record_draw_instructions");
        res_manager.wait_for(&compute_shader, render_instance).await;
        let mut record_indirect_compute_instructions_pipeline = ComputePipeline::new("Draw indirect instructions");
        if record_indirect_compute_instructions_pipeline
            .set_shader(&res_manager.get::<ShaderResource>(&compute_shader).unwrap().data.as_ref().unwrap().module)
            .add_bind_group(draw_indirect_desc_buffer_temporary.create_bind_group_layout(
                render_instance, BufferBindingType::Storage { read_only: true }, ShaderStages::COMPUTE).await)
            .add_bind_group(draw_indirect_desc_buffer.create_bind_group_layout(
                render_instance, BufferBindingType::Storage { read_only: false }, ShaderStages::COMPUTE).await)
            .add_bind_group(draw_indirect_data_buffer.create_bind_group_layout(
                render_instance, BufferBindingType::Storage { read_only: false }, ShaderStages::COMPUTE).await)
            .init(&render_instance).is_err() {
            error!("Failed to initialize compute pipeline.");
        }


        // ==== Camera buffer ====
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


        // ==== Render textures ====
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

            indirect_commands_buffer: indirect_commands_buffer,
            indirect_commands_buffer_bind_group,
            batch_commands_buffer,
            batch_commands_buffer_bind_group,
            draw_indirect_desc_buffer: draw_indirect_desc_buffer,
            draw_indirect_desc_buffer_bind_group,
            _draw_indirect_desc_buffer_temporary: draw_indirect_desc_buffer_temporary,
            draw_indirect_desc_buffer_temporary_bind_group_write,
            draw_indirect_desc_buffer_temporary_bind_group_read,
            draw_indirect_data_buffer,
            draw_indirect_data_buffer_bind_group,
            record_indirect_compute_pipeline,
            record_indirect_compute_instructions_pipeline,

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

        // Acquire render texture
        let render_texture: RenderTexture = match RenderInstance::get_current_texture(&render_instance) {
            RenderEvent::Redraw(render_texture) => render_texture,
            event => return event,
        };

        // Create draw batches
        let mut batch_references: Vec<(ResourceHandle, ResourceHandle)> = Vec::new();
        let mut draws_batches: Vec<IndirectBatch> = Vec::new();
        {
            trace!("Creating draw batches.");
            let _trace_draws = tracing::span!(tracing::Level::TRACE, "create_batches");
            
            // Create draw batch for first entity
            let mut batch_indices: Vec<(u32, u32)> = Vec::new();
            let first_entity = match world.get_entities_with_component::<RenderComponent>().iter().next() {
                Some(entity) => {
                    world.get_component::<RenderComponent>(*entity).unwrap()
                }
                None => return RenderEvent::None,
            };
            draws_batches.push(IndirectBatch {
                first: first_entity.id,
                count: 1,
                batch_index: 0,
                index_count: res_manager.get::<ModelResource>(&first_entity.model).unwrap().data.as_ref().unwrap().index_count as u32
            });
            batch_indices.push((first_entity.model.index as u32, first_entity.material.index as u32));
            batch_references.push((first_entity.model.clone(), first_entity.material.clone()));


            // Create draw batches for entities with render component
            let mut first_entity = true;
            for entity in world.get_entities_with_component::<RenderComponent>().iter() {
                // Ignore first entity
                if first_entity {
                    first_entity = false;
                    continue;
                }

                // Compare model and material with the last draw
                let entity_render = world.get_component::<RenderComponent>(*entity).unwrap();
                let entity_batch_index = match batch_indices.iter().position(|&pair| pair == (entity_render.model.index as u32, entity_render.material.index as u32)) {
                    Some(index) => index,
                    None => {
                        batch_indices.push((entity_render.model.index as u32, entity_render.material.index as u32));
                        batch_references.push((entity_render.model.clone(), entity_render.material.clone()));
                        batch_indices.len() - 1
                    }
                };
                let same_batch_index = draws_batches.last().unwrap().batch_index == entity_batch_index as u32;
                let contiguous = entity_render.id == (draws_batches.last().unwrap().first + draws_batches.last().unwrap().count);

                // Handle draw creation
                if same_batch_index && contiguous {
                    // Add entity to the last draw
                    let last_draw = draws_batches.last_mut().unwrap();
                    last_draw.count += 1;
                }
                else {
                    // Create a new draw
                    let new_draw = IndirectBatch {
                        first: entity_render.id,
                        count: 1,
                        batch_index: entity_batch_index as u32,
                        index_count: res_manager.get::<ModelResource>(&entity_render.model).unwrap().data.as_ref().unwrap().index_count as u32
                    };
                    draws_batches.push(new_draw);
                }
            }

            // Create draw batches for entities with render component instanced
            for entity in world.get_entities_with_component::<RenderComponentInstanced>().iter() {
                // Compare model and material with the last draw
                let entity_render = world.get_component::<RenderComponentInstanced>(*entity).unwrap();
                let entity_batch_index = match batch_indices.iter().position(|&pair| pair == (entity_render.model.index as u32, entity_render.material.index as u32)) {
                    Some(index) => index,
                    None => {
                        batch_indices.push((entity_render.model.index as u32, entity_render.material.index as u32));
                        batch_references.push((entity_render.model.clone(), entity_render.material.clone()));
                        batch_indices.len() - 1
                    }
                };
                let same_batch_index = draws_batches.last().unwrap().batch_index == entity_batch_index as u32;
                let contiguous = entity_render.ids.start == (draws_batches.last().unwrap().first + draws_batches.last().unwrap().count);

                // Handle draw creation
                if same_batch_index && contiguous {
                    // Add entity to the last draw
                    let last_draw = draws_batches.last_mut().unwrap();
                    last_draw.count += entity_render.ids.end - entity_render.ids.start;
                }
                else {
                    // Create a new draw
                    let new_draw = IndirectBatch {
                        first: entity_render.ids.start,
                        count: entity_render.ids.end - entity_render.ids.start,
                        batch_index: entity_batch_index as u32,
                        index_count: res_manager.get::<ModelResource>(&entity_render.model).unwrap().data.as_ref().unwrap().index_count as u32
                    };
                    draws_batches.push(new_draw);
                }
            }
        }

        // Fill batch commands buffer
        {
            trace!("Filling batch commands buffer.");
            let _trace_draws = tracing::span!(tracing::Level::TRACE, "fill_batch_commands");

            // Write batch commands
            self.batch_commands_buffer.map_mut(&render_instance, |mut view| {
                // Cast data to IndirectBatch
                let data = view.as_mut_ptr() as *mut IndirectBatch;

                // Write data
                for (i, draw) in draws_batches.iter().enumerate() {
                    unsafe {
                        *data.add(i) = *draw;
                    };
                }
            });
        }

        // Sort draw batches
        {
            trace!("Sorting draw batches.");
            let _trace_draws = tracing::span!(tracing::Level::TRACE, "sort_batches");

            // Sort draw batches
            draws_batches.sort_by(|a, b| a.batch_index.cmp(&b.batch_index));
        }

        // Create indirect commands
        {
            trace!("Create indirect commands.");
            let _trace_compute = tracing::span!(tracing::Level::TRACE, "create_indirect_commands");

            // Clear indirect commands data
            self.draw_indirect_data_buffer.map_mut(&render_instance, |mut view| {
                let data = view.as_mut_ptr() as *mut DrawIndirectData;
                unsafe {
                    *data = DrawIndirectData { descriptor_count: 0 as u32 };
                }
            });

            // Create command buffer
            let mut command_buffer = CommandBuffer::new(
                    &render_instance, "Create indirect commands").await;

            { // Create indirect commands list
                let mut compute_pass = command_buffer.create_compute_pass("Create indirect commands");

                // Set bind groups
                compute_pass.set_bind_group(0, &self.batch_commands_buffer_bind_group);
                compute_pass.set_bind_group(1, &self.draw_indirect_desc_buffer_temporary_bind_group_write);
                compute_pass.set_bind_group(2, &self.indirect_commands_buffer_bind_group);

                // Set pipeline
                if compute_pass.set_pipeline(&self.record_indirect_compute_pipeline).is_err() {
                    error!("Failed to set compute pipeline.");
                    return RenderEvent::None;
                }

                // Run compute shader
                if compute_pass.dispatch((draws_batches.len() as f32 / 256.0).ceil() as u32, 1, 1).is_err() {
                    error!("Failed to run compute shader.");
                    return RenderEvent::None;
                }
            }

            { // Create CPU indirect commands instructions 
                let mut compute_pass = command_buffer.create_compute_pass("Create indirect commands instructions");

                // Set bind groups
                compute_pass.set_bind_group(0, &self.draw_indirect_desc_buffer_temporary_bind_group_read);
                compute_pass.set_bind_group(1, &self.draw_indirect_desc_buffer_bind_group);
                compute_pass.set_bind_group(2, &self.draw_indirect_data_buffer_bind_group);

                // Set pipeline
                if compute_pass.set_pipeline(&self.record_indirect_compute_instructions_pipeline).is_err() {
                    error!("Failed to set compute pipeline.");
                    return RenderEvent::None;
                }

                // Run compute shader
                if compute_pass.dispatch((draws_batches.len() as f32 / 256.0).ceil() as u32, 1, 1).is_err() {
                    error!("Failed to run compute shader.");
                    return RenderEvent::None;
                }
            }

            // Submit command buffer
            command_buffer.submit(&render_instance);
        }

        // Render batches
        {
            trace!("Rendering batches.");
            let _trace_render = tracing::span!(tracing::Level::TRACE, "render_batches");

            // Read draw indirect data descriptor count
            let mut draw_indirect_desc_count = 0;
            self.draw_indirect_data_buffer.map(&render_instance, |view| {
                let data = view.as_ref().as_ptr() as *const DrawIndirectData;
                draw_indirect_desc_count = unsafe { *data }.descriptor_count;
            });

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
                let mut last_model = None;
                let mut last_material = None;

                // Map draw indirect descriptor buffer
                self.draw_indirect_desc_buffer.map(&render_instance, |view| {
                    let data = view.as_ref().as_ptr() as *const DrawIndexedIndirectDesc;

                    // Render entities
                    let mut it = 0;
                    let mut render_calls_count = 0;
                    while render_calls_count < draw_indirect_desc_count {
                        let draw = unsafe { *data.add(it) };
                        it += 1;
                        if draw.count == 0 { // Skip empty draws
                            continue;
                        }

                        // Get model
                        if let Some(model) = res_manager.get::<ModelResource>(&batch_references[draw.batch_index as usize].0) {
                            // Check if model is initialized
                            if !model.loaded() {
                                continue;
                            }

                            // Set model buffers
                            let batch_model = batch_references[draw.batch_index as usize].0.index;
                            if last_model != Some(batch_model) {
                                render_pass.set_vertex_buffer(0, &model.data.as_ref().unwrap().vertex_buffer);
                                render_pass.set_index_buffer(&model.data.as_ref().unwrap().index_buffer);
                                last_model = Some(batch_model);
                            }

                            // Get material
                            if let Some(material) = res_manager.get_mut::<MaterialResource>(&batch_references[draw.batch_index as usize].1) {
                                // Check if pipeline is initialized
                                if !material.data.as_ref().unwrap().pipeline.is_initialized() {
                                    continue;
                                }

                                // Set render pipeline
                                let batch_material = batch_references[draw.batch_index as usize].1.index;
                                if last_material != Some(batch_material) {
                                    if render_pass.set_pipeline(&material.data.as_ref().unwrap().pipeline).is_err() {
                                        error!("Failed to set pipeline for material {}.", material.label);
                                        continue;
                                    }
                                    last_material = Some(batch_material);
                                }

                                // Draw
                                render_pass
                                    .multi_draw_indexed_indirect(&self.indirect_commands_buffer, (draw.first as usize * std::mem::size_of::<DrawIndexedIndirectArgs>()) as u64, draw.count)
                                    .unwrap_or_else(|_| {
                                        error!("Failed to draw batch {:?}.", draw);
                                    });
                                
                                // Increment render calls count
                                render_calls_count += 1;
                            }
                        }
                    }
                });
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
