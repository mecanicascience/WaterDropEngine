//! An intermediate layer based on wgpu-rs for the creation of a game engine.
//! 
//! # Initialization
//! First, you need to create a new [RenderInstance] linked to the window.
//! 
//! ## Render Instance
//! The [RenderInstance] is the main object that is used to create resources and pipelines.
//! It is created by calling the [RenderInstance::new] function.
//! 
//! ```rust
//! // Create a new instance linked to the window
//! let mut (...) = RenderInstance::new("WaterDropEngine", &app).await;
//! let mut (...) = RenderInstance::setup_surface(...);
//! 
//! // Get current texture
//! let render_texture = RenderInstance::get_current_texture(&instance);
//! 
//! // Render
//! // ...
//! 
//! // Present texture
//! RenderInstance::present(instance.render_texture);
//! 
//! // Resize the surface
//! // This must be called when the window is resized
//! RenderInstance::resize(instance.device, instance.surface, instance.surface_config);
//! ```
//! 
//! # Register resources
//! Different resources can be created and used in the shaders, such as buffers and textures.
//! 
//! ## Buffer
//! A [Buffer] is a block of memory that can be used to store data that can be used in shaders.
//! 
//! ```rust
//! // Create a new buffer
//! let mut buffer = Buffer::new(&instance, "Buffer label", 1024, BufferUsage::Vertex, None);
//! 
//! // Copy data to the buffer
//! // Note that the buffer must have the COPY_DST usage.
//! buffer.copy_from_buffer(&instance, &buffer);
//! 
//! // Copy data to the buffer from a texture
//! // Note that the buffer must have the COPY_DST usage.
//! buffer.copy_from_texture(&instance, &texture);
//! 
//! // Write data to the buffer starting at 16 bytes
//! // Note that the buffer must have the COPY_DST usage.
//! buffer.write(&instance, bytemuck::cast_slice(&[data]), 16);
//! 
//! // Map the buffer and read the data
//! // Note that the buffer must have the MAP_READ usage.
//! buffer.map_read(&instance, |data| {
//!     let data = bytemuck::cast_slice(data);
//!     // ...
//! });
//! 
//! // Map the buffer and write the data
//! // Note that the buffer must have the MAP_WRITE usage.
//! buffer.map_write(&instance, |data| {
//!     let data = bytemuck::cast_slice_mut(data);
//!     // ...
//! });
//! ```
//! 
//! ## Texture
//! A [Texture] is a 2D image that can be used as a render target or a texture in a shader.
//! 
//! ```rust
//! // Create a new texture
//! let texture = Texture::new(&instance,
//!     "Texture Label", (1024, 1024), TextureFormat::Rgba8Unorm,
//!     TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC);
//! 
//! // Copy buffer to texture.
//! // Note that the texture must have the COPY_DST usage.
//! texture.copy_from_buffer(&instance, &buffer, false);
//! 
//! // Copy texture to texture
//! // Note that the texture must have the COPY_DST and COPY_SRC usages.
//! texture.copy_from_texture(&instance, &texture, (1024, 1024));
//! ```
//! 
//! 
//! # Render pipeline creation
//! The render pipeline is used to draw primitives to a texture.
//! 
//! ## Bind Group
//! First, you need to create a [BindGroupBuilder] describing the resources that will be used in the pipeline.
//! Then you need to generate a [BindGroup] from the bind group builder that will be used in the pipeline.
//! The layout of the bind group must match the layout of the bind group in the shader.
//! 
//! ```rust
//! // Create a new bind group builder
//! let mut bind_group_builder = BindGroupBuilder::new("Bind Group");
//! 
//! // Add a buffer to the bind group
//! bind_group_builder.add_buffer(0, &buffer, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform);
//! 
//! // Add a texture to the bind group. Note that the binding is incremented by 1.
//! bind_group_builder.add_texture(1, &texture, wgpu::ShaderStages::FRAGMENT);
//! 
//! // Build the bind group
//! // Note that this will move the bind group builder, so we need to clone it if we want to use it again.
//! let bind_group = BindGroup::new(&instance, bind_group_builder.clone());
//! ```
//! 
//! ## Render pipeline
//! The vertices of a mesh in a render pipeline are described by their position, texture UV, and normal, as described in the [Vertex] struct.
//! ```rust
//! Vertex {
//!    position: [5.0, 2.0, -2.0],
//!    uv:       [1.0, 0.0],      // Must be between 0.0 and 1.0
//!    normal:   [0.0, 1.0, 0.0], // Must be normalized
//! };
//! ```
//! 
//! In the shader, the vertex is described as follows:
//!  - `layout(location = 0) in vec3 a_position` for the position.
//!  - `layout(location = 1) in vec2 a_uv` for the texture UV.
//!  - `layout(location = 2) in vec3 a_normal` for the normal.
//! 
//! You can create a render pipeline by calling the [RenderPipeline] constructor.
//! First, you describe the different stages of the pipeline, such as the vertex buffer, the fragment shader, and the bind group.
//! Then you create the render pipeline.
//! ```rust
//! let mut pipeline = WRenderPipeline::new("...");
//! pipeline
//!     .set_shader(include_str!("[...].vert"), ShaderType::Vertex)   // Set the vertex shader
//!     .set_shader(include_str!("[...].frag"), ShaderType::Fragment) // Set the fragment shader
//!     .set_topology(Topology::LineList)            // Change the primitive topology
//!     .set_depth_stencil()                         // Enable depth and stencil
//!     .add_push_constant(ShaderType::Vertex, 0, 4) // Say that we will provide push constant at offset 0 with size 4
//!     .add_bind_group(bind_group_layout)           // Say that we will use a bind group
//!     .init(&instance);                            // Initialize the pipeline
//! 
//! if pipeline.is_initialized() {
//!    // Use the pipeline
//!    let pipeline = pipeline.get_pipeline().unwrap();
//!    let layout = pipeline.get_layout().unwrap();
//! 
//!    // ...
//! }
//! ```
//! 
//! ## Command Buffer and Render Pass
//! In the draw loop, you need to create a [CommandBuffer] that will be used to register GPU commands.
//! 
//! Then you create a [RenderPass] that will be used to draw primitives to a color texture, and optionally a depth texture.
//! You can have multiple render passes in a command buffer.
//! You can finally draw primitives in the render pass using the different [RenderPass] methods.
//! 
//! ```rust
//! // Create a new command buffer
//! let mut command_buffer = WCommandBuffer::new(&instance, "Command Buffer");
//! 
//! {
//!     // Create a render pass
//!     let mut render_pass = command_buffer.create_render_pass(
//!         "Render Pass", &color_texture,
//!         Some(Operations { load: LoadOp::Clear(Color::BLACK), store: StoreOp::Store }),
//!         Some(&depth_texture));
//! 
//!     // Set render pass dependencies
//!     render_pass
//!         .set_scissor_rect(0, 0, 800, 600) // Set the scissor rect to (0, 0) with width 800 and height 600
//!         .set_vertex_buffer(vertex_buffer) // Set the vertex buffer of the current render pass
//!         .set_index_buffer(index_buffer);  // Set the index buffer of the current render pass
//! 
//!     render_pass
//!         .set_pipeline(&render_pipeline)  // Set the pipeline of the render pass. The pipeline must be initialized.
//!         .set_push_constants(ShaderType::Vertex, bytemuck::cast_slice(&[...])) // Set push constants values
//!         .set_bind_group(0, &bind_group); // Set bind group at binding 0
//! 
//!     // You can then render primitives using the different methods of the render pass.
//!     // The following methods are available:
//! 
//!     // Render primitives
//!     // The first parameter is the range of vertices to draw, and the second parameter is the range of instances to draw.
//!     render_pass.draw(first_vertex..last_vertex, first_instance_index..last_instance_index);
//! 
//!     // Render indexed
//!     // The first parameter is the range of indices to draw, and the second parameter is the range of instances to draw.
//!     render_pass.draw_indexed(first_index..last_index, first_instance_index..last_instance_index);
//! 
//!     // Draw primitives from the active vertex buffers.
//!     // The draw is indirect, meaning the draw arguments are read from a buffer.
//!     // The first parameter is the offset (the starting command in the buffer), and the second parameter is the number of commands to execute.
//!     render_pass.multi_draw_indirect(indirect_buffer, first_draw_command_index, draw_commands_count);
//! 
//!     // Draw primitives from the active vertex buffers as indexed triangles.
//!     // The draw is indirect, meaning the draw arguments are read from a buffer.
//!     // The first parameter is the offset (the starting command in the buffer), and the second parameter is the number of commands to execute.
//!     render_pass.multi_draw_indexed_indirect(indirect_buffer, first_draw_command_index, draw_commands_count);
//! 
//!     // Dropping the render pass will end the render pass and submit it to the command buffer.
//! }
//! 
//! // Submit the command buffer to the GPU
//! command_buffer.submit(&instance);
//! ```
//! 
//! 
//! # Compute pipeline creation
//! The compute pipeline is used to run compute shaders on the GPU.
//! 
//! ## Bind Group
//! The bind group is created in the same way as for the render pipeline (see above).
//! 
//! ## Compute pipeline
//! You can create a compute pipeline by calling the [ComputePipeline] constructor.
//! First, you describe the different stages of the pipeline, such as the compute shader and the bind group.
//! Then you create the compute pipeline.
//! ```rust
//! // Create a new compute pipeline
//! let mut pipeline = WComputePipeline::new("Compute Pipeline");
//! pipeline
//!    .set_shader(include_str!("[...].comp"))   // Set the compute shader
//!    .add_push_constant(4)                     // Say that we will provide push constant at offset 0 with size 4
//!    .add_bind_group(bind_group.layout)        // Say that we will use a bind group
//!    .init(&instance);                         // Initialize the pipeline
//! 
//! // Check if the pipeline is initialized
//! if pipeline.is_initialized() {
//!    // Get the compute pipeline
//!    let compute_pipeline = pipeline.get_pipeline().unwrap();
//!    
//!    // Get the pipeline layout
//!    let layout = pipeline.get_layout().unwrap();
//! }
//! ```
//! 
//! ## Compute pass
//! In the compute pass, you can set the pipeline, push constants, and bind groups.
//! You can then dispatch the compute pass.
//! 
//! ```rust
//! // Create a new compute pass
//! let mut compute_pass = WComputePass::new("Compute pass");
//! 
//! // Set the pipeline dependencies
//! compute_pass
//!     .set_pipeline(&compute_pipeline)  // Set the pipeline of the compute pass. The pipeline must be initialized.
//!     .set_push_constants(bytemuck::cast_slice(&[...]))  // Set push constants values
//!     .set_bind_group(0, &bind_group);  // Set bind group at binding 0
//! 
//! // Run compute pass on the GPU on a given number of workgroups (x, y, z)
//! compute_pass.dispatch(x: ..., y: ..., z: ...);
//! ```
//! 
//! 
//! [RenderInstance]: instance/struct.RenderInstance.html
//! [RenderInstance::new]: instance/struct.RenderInstance.html#method.new
//! [Buffer]: buffer/struct.Buffer.html
//! [Texture]: texture/struct.Texture.html
//! [RenderPipeline]: render_pipeline/struct.RenderPipeline.html
//! [BindGroupBuilder]: bind_group/struct.BindGroupBuilder.html
//! [BindGroup]: bind_group/struct.BindGroup.html
//! [Vertex]: vertex/struct.Vertex.html
//! [CommandBuffer]: command_buffer/struct.CommandBuffer.html
//! [RenderPass]: render_pass/struct.RenderPass.html
//! [ComputePipeline]: compute_pipeline/struct.ComputePipeline.html
//! [ComputePass]: compute_pass/struct.ComputePass.html
pub mod instance;
pub mod vertex;
pub mod bind_group;
pub mod render_pipeline;
pub mod compute_pipeline;
pub mod texture;
pub mod render_pass;
pub mod compute_pass;
pub mod buffer;
pub mod command_buffer;
