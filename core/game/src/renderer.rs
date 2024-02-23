use tracing::{debug, error, trace, warn};
use wde_ecs::{CameraComponent, CameraUniform, EntityIndex, RenderComponent, RenderComponentInstanced, RenderComponentSSBODynamic, RenderComponentSSBOStatic, TransformComponent, TransformUniform, World, MAX_ENTITIES};
use wde_resources::{MaterialResource, ModelResource, Resource, ResourceHandle, ResourcesManager, ShaderResource};
use wde_wgpu::{BindGroup, BindGroupBuilder, Buffer, BufferBindingType, BufferUsage, Color, CommandBuffer, ComputePipeline, DrawIndexedIndirectArgs, LoadOp, Operations, RenderEvent, RenderInstance, RenderTexture, ShaderStages, StoreOp, Texture, TextureUsages};

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

    // Sort batches
    sort_batches: ComputePipeline,

    // Draw indirect commands
    batch_write_bg: BindGroup,
    batch_read_bg: BindGroup,
    indirect_desc_bg: BindGroup,
    indirect_desc_commands_bg: BindGroup,
    batch_buffer: Buffer,
    indirect_desc: Buffer,
    indirect_buffer: Buffer,
    indirect_compute: ComputePipeline,

    // Draw indirect description
    indirect_desc_tmp_read_bg: BindGroup,
    batch_buffer_indices: Buffer,
    indirect_data: Buffer,
    _indirect_desc_tmp: Buffer,
    indirect_compute_instructions: ComputePipeline,
    
    // Camera buffer
    camera_buffer: Buffer,
    camera_buffer_bg: BindGroup,

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
        let objects = Buffer::new(
            &render_instance,
            "Object matrices SSBO",
            std::mem::size_of::<TransformUniform>() * MAX_ENTITIES as usize,
            BufferUsage::STORAGE | BufferUsage::MAP_WRITE,
            None);

        // Create object matrices SSBO bind group
        let mut objects_bg_build = BindGroupBuilder::new("Objects matrices SSBO");
        objects_bg_build
            .add_buffer(0, &objects, ShaderStages::VERTEX, BufferBindingType::Storage { read_only: true });


        // ==== Sort batches ====
        // Create buffers
        let batch_buffer = Buffer::new(
            &render_instance,
            "Batch commands buffer",
            std::mem::size_of::<IndirectBatch>() * MAX_INDIRECT_COMMANDS as usize,
            BufferUsage::MAP_WRITE | BufferUsage::STORAGE,
            None);
        let batch_buffer_indices = Buffer::new(
            &render_instance,
            "Batch commands buffer indices",
            std::mem::size_of::<u32>() * MAX_INDIRECT_COMMANDS as usize,
            BufferUsage::MAP_WRITE | BufferUsage::STORAGE,
            None);

        // Create bind groups
        let mut batch_bg_build_write = BindGroupBuilder::new("Batch commands buffer");
        batch_bg_build_write
            .add_buffer(0, &batch_buffer_indices, ShaderStages::COMPUTE, BufferBindingType::Storage { read_only: false })
            .add_buffer(1, &batch_buffer, ShaderStages::COMPUTE, BufferBindingType::Storage { read_only: true });

        // Create compute pipeline
        let shader = res_manager.load::<ShaderResource>("compute/sort_batches");
        res_manager.wait_for(&shader, render_instance).await;
        let mut sort_batches = ComputePipeline::new("Sort batches");
        if sort_batches
            .set_shader(&res_manager.get::<ShaderResource>(&shader).unwrap().data.as_ref().unwrap().module)
            .add_push_constant(std::mem::size_of::<[u32; 2]>() as u32)
            .add_bind_group(BindGroup::new(&render_instance, batch_bg_build_write.clone()))
            .init(&render_instance).is_err() { error!("Failed to initialize compute pipeline."); }


        // ==== Indirect draw commands ====
        // Create buffers
        let indirect_desc_tmp = Buffer::new(
            &render_instance,
            "Draw indirect descriptor buffer temporary",
            std::mem::size_of::<DrawIndexedIndirectDesc>() * MAX_INDIRECT_COMMANDS as usize,
            BufferUsage::STORAGE,
            None);
        let indirect_buffer = Buffer::new(
            &render_instance,
            "Draw indirect commands buffer",
            std::mem::size_of::<DrawIndexedIndirectArgs>() * MAX_INDIRECT_COMMANDS as usize,
            BufferUsage::INDIRECT | BufferUsage::STORAGE,
            None);

        // Create bind groups
        let mut batch_bg_build_read = BindGroupBuilder::new("Batch commands buffer");
        batch_bg_build_read
            .add_buffer(0, &batch_buffer_indices, ShaderStages::COMPUTE, BufferBindingType::Storage { read_only: true })
            .add_buffer(1, &batch_buffer, ShaderStages::COMPUTE, BufferBindingType::Storage { read_only: true });
        let mut indirect_desc_commands_bg_build = BindGroupBuilder::new("Draw indirect descriptor buffer temporary write");
        indirect_desc_commands_bg_build
            .add_buffer(0, &indirect_desc_tmp, ShaderStages::COMPUTE, BufferBindingType::Storage { read_only: false })
            .add_buffer(1, &indirect_buffer, ShaderStages::COMPUTE, BufferBindingType::Storage { read_only: false });
        
        // Create compute pipeline
        let shader = res_manager.load::<ShaderResource>("compute/record_draw_commands");
        res_manager.wait_for(&shader, render_instance).await;
        let mut indirect_compute = ComputePipeline::new("Draw indirect");
        if indirect_compute
            .set_shader(&res_manager.get::<ShaderResource>(&shader).unwrap().data.as_ref().unwrap().module)
            .add_bind_group(BindGroup::new(&render_instance, batch_bg_build_read.clone()))
            .add_bind_group(BindGroup::new(&render_instance, indirect_desc_commands_bg_build.clone()))
            .init(&render_instance).is_err() { error!("Failed to initialize compute pipeline."); }


        // ==== Indirect draw description ====
        // Create buffers
        let indirect_desc = Buffer::new(
            &render_instance,
            "Draw indirect descriptor",
            std::mem::size_of::<DrawIndexedIndirectDesc>() * MAX_INDIRECT_COMMANDS as usize,
            BufferUsage::STORAGE | BufferUsage::MAP_READ,
            None);
        let indirect_data = Buffer::new(
            &render_instance,
            "Draw indirect data",
            std::mem::size_of::<DrawIndirectData>() as usize,
            BufferUsage::STORAGE | BufferUsage::MAP_READ | BufferUsage::MAP_WRITE,
            None);

        // Create bind groups
        let mut indirect_desc_tmp_read_bg_build = BindGroupBuilder::new("Draw indirect descriptor buffer temporary read");
        indirect_desc_tmp_read_bg_build
            .add_buffer(0, &indirect_desc_tmp, ShaderStages::COMPUTE, BufferBindingType::Storage { read_only: true });
        let mut indirect_desc_bg_build = BindGroupBuilder::new("Draw indirect descriptor buffer");
        indirect_desc_bg_build
            .add_buffer(0, &indirect_desc, ShaderStages::COMPUTE, BufferBindingType::Storage { read_only: false })
            .add_buffer(1, &indirect_data, ShaderStages::COMPUTE, BufferBindingType::Storage { read_only: false });

        // Create compute pipeline
        let shader = res_manager.load::<ShaderResource>("compute/record_draw_instructions");
        res_manager.wait_for(&shader, render_instance).await;
        let mut indirect_compute_instructions = ComputePipeline::new("Draw indirect instructions");
        if indirect_compute_instructions
            .set_shader(&res_manager.get::<ShaderResource>(&shader).unwrap().data.as_ref().unwrap().module)
            .add_bind_group(BindGroup::new(&render_instance, indirect_desc_tmp_read_bg_build.clone()))
            .add_bind_group(BindGroup::new(&render_instance, indirect_desc_bg_build.clone()))
            .init(&render_instance).is_err() { error!("Failed to initialize compute pipeline."); }


        // ==== Camera buffer ====
        // Create camera uniform buffer
        let camera_buffer = Buffer::new(
            &render_instance,
            "Camera buffer",
            std::mem::size_of::<CameraUniform>(),
            wde_wgpu::BufferUsage::UNIFORM | wde_wgpu::BufferUsage::COPY_DST,
            None);

        // Create camera buffer bind group
        let mut camera_buffer_bg_build = BindGroupBuilder::new("Camera buffer");
        camera_buffer_bg_build
            .add_buffer(0, &camera_buffer, ShaderStages::VERTEX, BufferBindingType::Uniform);
        let camera_buffer_bg = BindGroup::new(&render_instance, camera_buffer_bg_build);


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
            objects_bind_group: BindGroup::new(&render_instance, objects_bg_build),
            objects,

            sort_batches,

            batch_write_bg: BindGroup::new(&render_instance, batch_bg_build_write),
            batch_read_bg: BindGroup::new(&render_instance, batch_bg_build_read),
            indirect_desc_bg: BindGroup::new(&render_instance, indirect_desc_bg_build),
            indirect_desc_commands_bg: BindGroup::new(&render_instance, indirect_desc_commands_bg_build),
            batch_buffer,
            batch_buffer_indices,
            indirect_desc,
            indirect_buffer,
            indirect_compute,

            indirect_desc_tmp_read_bg: BindGroup::new(&render_instance, indirect_desc_tmp_read_bg_build),
            _indirect_desc_tmp: indirect_desc_tmp,
            indirect_data,
            indirect_compute_instructions,

            camera_buffer,
            camera_buffer_bg,

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

                // Initialize pipeline
                if let Some(material) = res_manager.get_mut::<MaterialResource>(&render_component.material) {
                    // Create layouts
                    let mut camera_buffer_bg_build = BindGroupBuilder::new("Camera buffer");
                    camera_buffer_bg_build
                        .add_buffer(0, &self.camera_buffer, ShaderStages::VERTEX, BufferBindingType::Uniform);

                    let mut objects_bind_group_layout = BindGroupBuilder::new("Objects matrices SSBO");
                    objects_bind_group_layout
                        .add_buffer(0, &self.objects, ShaderStages::VERTEX, BufferBindingType::Storage { read_only: true });

                    material.data.as_mut().unwrap().pipeline
                        .add_bind_group(BindGroup::new(&render_instance, camera_buffer_bg_build.clone()).layout)
                        .add_bind_group(BindGroup::new(&render_instance, objects_bind_group_layout.clone()).layout)
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

                // Initialize pipeline
                if let Some(material) = res_manager.get_mut::<MaterialResource>(&render_component.material) {
                    // Create layouts
                    let mut camera_buffer_bg_build = BindGroupBuilder::new("Camera buffer");
                    camera_buffer_bg_build
                        .add_buffer(0, &self.camera_buffer, ShaderStages::VERTEX, BufferBindingType::Uniform);

                    let mut objects_bind_group_layout = BindGroupBuilder::new("Objects matrices SSBO");
                    objects_bind_group_layout
                        .add_buffer(0, &self.objects, ShaderStages::VERTEX, BufferBindingType::Storage { read_only: true });

                    material.data.as_mut().unwrap().pipeline
                        .add_bind_group(BindGroup::new(&render_instance, camera_buffer_bg_build.clone()).layout)
                        .add_bind_group(BindGroup::new(&render_instance, objects_bind_group_layout.clone()).layout)
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
    /// * `editor_handler` - The editor handler
    /// * `render_texture` - The render texture
    #[tracing::instrument]
    pub async fn render(&self, render_instance: &RenderInstance<'_>, world: &World, res_manager: &ResourcesManager, render_texture: &RenderTexture) -> RenderEvent {
        debug!("Starting render.");

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
            self.batch_buffer.map_mut(&render_instance, |mut view| {
                // Cast data to IndirectBatch
                let data = view.as_mut_ptr() as *mut IndirectBatch;

                // Write data
                for (i, draw) in draws_batches.iter().enumerate() {
                    unsafe {
                        *data.add(i) = *draw;
                    };
                }
            });

            // Write batch commands indices
            self.batch_buffer_indices.map_mut(&render_instance, |mut view| {
                // Cast data to u32
                let data = view.as_mut_ptr() as *mut u32;

                // Write data
                for (i, _pair) in draws_batches.iter().enumerate() {
                    unsafe {
                        *data.add(i) = i as u32;
                    };
                }
            });
        }

        // Create indirect commands
        {
            trace!("Create indirect commands.");
            let _trace_compute = tracing::span!(tracing::Level::TRACE, "create_indirect_commands");

            // Clear indirect commands data
            self.indirect_data.map_mut(&render_instance, |mut view| {
                let data = view.as_mut_ptr() as *mut DrawIndirectData;
                unsafe {
                    *data = DrawIndirectData { descriptor_count: 0 as u32 };
                }
            });

            // Create command buffer
            let mut command_buffer = CommandBuffer::new(
                    &render_instance, "Create indirect commands").await;

            { // Sort batches
                let mut compute_pass = command_buffer.create_compute_pass("Sort batches");

                // Set bind groups
                compute_pass.set_bind_group(0, &self.batch_write_bg);

                // Set pipeline
                if compute_pass.set_pipeline(&self.sort_batches).is_err() {
                    error!("Failed to set compute pipeline.");
                    return RenderEvent::None;
                }

                for i in 0..draws_batches.len() {
                    // Set push constants
                    let push_constants = [draws_batches.len() as u32, (i % 2) as u32];
                    compute_pass.set_push_constants(bytemuck::cast_slice(&push_constants));

                    // Run compute shader
                    if compute_pass.dispatch((draws_batches.len() as f32 / 256.0).ceil() as u32, 1, 1).is_err() {
                        error!("Failed to run compute shader.");
                        return RenderEvent::None;
                    }
                }
            }

            { // Create indirect commands list
                let mut compute_pass = command_buffer.create_compute_pass("Create indirect commands");

                // Set bind groups
                compute_pass.set_bind_group(0, &self.batch_read_bg);
                compute_pass.set_bind_group(1, &self.indirect_desc_commands_bg);

                // Set pipeline
                if compute_pass.set_pipeline(&self.indirect_compute).is_err() {
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
                compute_pass.set_bind_group(0, &self.indirect_desc_tmp_read_bg);
                compute_pass.set_bind_group(1, &self.indirect_desc_bg);

                // Set pipeline
                if compute_pass.set_pipeline(&self.indirect_compute_instructions).is_err() {
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
            self.indirect_data.map(&render_instance, |view| {
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
                render_pass.set_bind_group(0, &self.camera_buffer_bg);
                render_pass.set_bind_group(1, &self.objects_bind_group);

                // Last model and material
                let mut last_model = None;
                let mut last_material = None;

                // Map draw indirect descriptor buffer
                self.indirect_desc.map(&render_instance, |view| {
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
                                    .multi_draw_indexed_indirect(&self.indirect_buffer, (draw.first as usize * std::mem::size_of::<DrawIndexedIndirectArgs>()) as u64, draw.count)
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
        // Update camera component
        let mut camera_component = world.get_component::<CameraComponent>(camera).unwrap().clone();
        let surface_config = render_instance.surface_config.as_ref().unwrap();
        camera_component.aspect = surface_config.width as f32 / surface_config.height as f32;

        // Create camera uniform
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.world_to_screen = CameraUniform::get_world_to_screen(
            camera_component,
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
