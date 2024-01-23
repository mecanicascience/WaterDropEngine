use tokio::sync::mpsc;
use tracing::{span, Level};
use wde_ecs::{World, TransformComponent, LabelComponent, RenderComponentDynamic, TransformUniform, CameraUniform, CameraComponent};
use wde_logger::{info, throw, trace, debug};
use wde_editor_interactions::EditorHandler;
use wde_math::{QUATF_IDENTITY, ONE_VEC3F, Vec3f};
use wde_resources::{ResourcesManager, ModelResource, ShaderResource};
use wde_wgpu::{LoopEvent, Window, RenderInstance, RenderEvent, CommandBuffer, LoadOp, Operations, StoreOp, Color, RenderPipeline, Event, WindowEvent, ShaderType, Buffer, ShaderStages};

pub struct App {}

impl App {
    pub async fn new() -> Self {
        // Configure environment if on debug mode
        if cfg!(debug_assertions) {
            // Set the RUST_BACKTRACE environment variable to 1
            std::env::set_var("RUST_BACKTRACE", "0");
        }
        debug!("======== Starting engine ========");


        // ======== WINDOW CREATION ========
        let _window_creation_span = span!(Level::INFO, "window_creation").entered();
        // Create window
        let mut window = Window::new(800, 600, "WaterDropEngine");

        // Create channel to send the window to the runtime
        let (window_t, mut window_r) = mpsc::unbounded_channel();

        // Run window
        trace!("Starting window.");
        let (event_t, mut event_r) = mpsc::unbounded_channel();
        let (event_relay_t, mut event_relay_r) = mpsc::unbounded_channel();
        let window_join = std::thread::spawn(move || {
            // Create event loop
            let event_loop = window.create();
            let window_index = window.run(&event_loop);

            // Send window to another channel
            window_t.send(window).unwrap();

            // Run event loop
            event_loop.run(move |event, elwt| {
                // Handle event
                match event {
                    // Close window when the close button is pressed
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } if window_id == window_index => {
                        info!("Closing window.");
                        event_t.send(LoopEvent::Close).unwrap_or_else(|e| {
                            throw!("Failed to send close event : {}.", e);
                        });
                        elwt.exit();
                    },

                    // Resize window when requested
                    Event::WindowEvent {
                        event: WindowEvent::Resized(size),
                        window_id,
                    } if window_id == window_index => {
                        if event_relay_r.try_recv().is_ok() {
                            let _event_loop_wait_span = span!(Level::INFO, "event_loop_wait").entered();
                            event_t.send(LoopEvent::Resize(size.width, size.height)).unwrap_or_else(|e| {
                                throw!("Failed to send resize event : {}.", e);
                            });
                        }
                    },

                    // Ask for redraw when all events are processed
                    Event::AboutToWait => {
                        if event_relay_r.try_recv().is_ok() {
                            let _event_loop_wait_span = span!(Level::INFO, "event_loop_wait").entered();
                            event_t.send(LoopEvent::Redraw).unwrap_or_else(|e| {
                                throw!("Failed to send redraw event : {}.", e);
                            });
                        }
                    },

                    // Redraw window when requested
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => {
                        if event_relay_r.try_recv().is_ok() {
                            let _event_loop_wait_span = span!(Level::INFO, "event_loop_wait").entered();
                            event_t.send(LoopEvent::Redraw).unwrap_or_else(|e| {
                                throw!("Failed to send redraw event : {}.", e);
                            });
                        }
                    },

                    // Ignore other events
                    _ => ()
                }
            }).unwrap_or_else(|e| {
                throw!("Failed to run event loop : {:?}.", e);
            });
        });
        drop(_window_creation_span);


        // ======== ENGINE INITIALIZATION ========
        let _engine_initialization_span = span!(Level::INFO, "engine_initialization").entered();

        // Create editor handler
        let mut editor_handler: Option<EditorHandler> = if cfg!(debug_assertions) != true { None } else {
            match EditorHandler::new() {
                Ok(h) => if h.started() { Some(h) } else { None }
                Err(_) => None
            }
        };

        // Create resource manager
        let mut res_manager = ResourcesManager::new();
        
        // Wait for window
        let window = window_r.recv().await.unwrap();

        // Create render instance
        let render_instance = RenderInstance::new("WaterDropEngine", Some(&window)).await;

        // Handle editor messages and push new frame
        if editor_handler.is_some() {
            let editor = editor_handler.as_mut().unwrap();
            match editor.process() {
                Ok(_) => {
                    // Set last frame
                    let r = rand::random::<u32>();
                    let _ = editor.set_current_frame(format!("Hello {} world.", r).as_bytes());
                },
                Err(_) => {}
            }
        }
        drop(_engine_initialization_span);


        // ======== WORLD CONTENT ========
        let _world_content_span = span!(Level::INFO, "world_content").entered();

        // Create world
        let mut world = World::new();
        world
            .register_component::<LabelComponent>()
            .register_component::<TransformComponent>()
            .register_component::<RenderComponentDynamic>();

        // Create camera
        let camera = match world.create_entity() {
            Some(e) => e,
            None => throw!("Failed to create entity. No more entity slots available."),
        };
        world
            .add_component(camera, LabelComponent { label : "Camera".to_string() }).unwrap()
            .add_component(camera, TransformComponent {
                position: Vec3f { x: 0.0, y: 0.0, z: 1.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F
            }).unwrap();

        // Create camera uniform buffer
        let mut camera_buffer = Buffer::new(
            &render_instance,
            "Camera buffer",
            std::mem::size_of::<CameraUniform>(),
            wde_wgpu::BufferUsage::UNIFORM | wde_wgpu::BufferUsage::COPY_DST,
            None);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.world_to_screen = CameraUniform::get_world_to_screen(
            CameraComponent { aspect: 16.0 / 9.0, fovy: 60.0, znear: 0.1, zfar: 1000.0 },
            world.get_component::<TransformComponent>(camera).unwrap().clone()
        ).into();
        camera_buffer.write(&render_instance, bytemuck::cast_slice(&[camera_uniform]), 0);

        // Create camera uniform buffer bind group layout
        let camera_buffer_bind_group_layout = camera_buffer.create_bind_group_layout(
            &render_instance,
            wde_wgpu::BufferBindingType::Uniform,
            ShaderStages::VERTEX).await;
        let camera_buffer_bind_group = camera_buffer.create_bind_group(
            &render_instance,
            wde_wgpu::BufferBindingType::Uniform,
            ShaderStages::VERTEX).await;


        // Create big model
        let big_model = match world.create_entity() {
            Some(e) => e,
            None => throw!("Failed to create entity. No more entity slots available."),
        };

        // Set model to big model
        world
            .add_component(big_model, LabelComponent { label : "Big model".to_string() }).unwrap()
            .add_component(big_model, TransformComponent {
                position: Vec3f { x: 0.0, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.3
            }).unwrap()
            .add_component(big_model, RenderComponentDynamic {
                model: res_manager.load::<ModelResource>("models/lost_empire.obj")
            }).unwrap();


        // Create cube
        let cube = match world.create_entity() {
            Some(e) => e,
            None => throw!("Failed to create entity. No more entity slots available."),
        };

        // Set model to cube
        world
            .add_component(cube, LabelComponent { label : "Cube".to_string() }).unwrap()
            .add_component(cube, TransformComponent {
                position: Vec3f { x: -0.5, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.3
            }).unwrap()
            .add_component(cube, RenderComponentDynamic {
                model: res_manager.load::<ModelResource>("models/cube.obj")
            }).unwrap();

        // Create default uniform buffer
        let mut model_buffer = Buffer::new(
            &render_instance,
            "Cube buffer",
            std::mem::size_of::<TransformUniform>(),
            wde_wgpu::BufferUsage::UNIFORM | wde_wgpu::BufferUsage::COPY_DST,
            None);
        let transform_uniform = TransformUniform::new(
            world.get_component::<TransformComponent>(cube).unwrap().clone()
        );
        model_buffer.write(&render_instance, bytemuck::cast_slice(&[transform_uniform]), 0);
        let model_buffer_bind_group_layout = model_buffer.create_bind_group_layout(
            &render_instance,
            wde_wgpu::BufferBindingType::Uniform,
            ShaderStages::VERTEX).await;
        let model_buffer_bind_group = model_buffer.create_bind_group(
            &render_instance,
            wde_wgpu::BufferBindingType::Uniform,
            ShaderStages::VERTEX).await;



        // Create shaders
        let vertex_shader_handle = res_manager.load::<ShaderResource>("shaders/vertex.wgsl");
        let fragment_shader_handle = res_manager.load::<ShaderResource>("shaders/frag.wgsl");

        // Wait for shaders to load
        res_manager.wait_for(&vertex_shader_handle, &render_instance).await;
        res_manager.wait_for(&fragment_shader_handle, &render_instance).await;

        // Create default render pipeline
        let mut render_pipeline = RenderPipeline::new("Main Render");
        let _ = render_pipeline
            .set_shader(&res_manager.get::<ShaderResource>(&vertex_shader_handle).unwrap().data.as_ref().unwrap().module, ShaderType::Vertex)
            .set_shader(&res_manager.get::<ShaderResource>(&fragment_shader_handle).unwrap().data.as_ref().unwrap().module, ShaderType::Fragment)
            .add_bind_group(camera_buffer_bind_group_layout)
            .add_bind_group(model_buffer_bind_group_layout)
            .init(&render_instance).await;

        // End of world content
        drop(_world_content_span);
            

        // ======== MAIN LOOP ========
        loop {
            let _next_frame_span = span!(Level::INFO, "next_frame").entered();
            debug!("\n\n\n======== Next frame ========");

            // ====== Wait for next render event ======
            {
                let _next_frame_wait_span = span!(Level::INFO, "next_frame_wait").entered();
                trace!("Waiting for next frame.");
                let _ = event_relay_t.send(());

                // Wait for next event
                let ev = event_r.recv().await;
                if ev.is_none() { break; }
                match ev.unwrap() {
                    LoopEvent::Close => { break; },
                    LoopEvent::Redraw => { },
                    LoopEvent::Resize(_, _) => { continue; },
                }
                trace!("Handling next frame.");
            }


            // ====== Update world ======
            {
                let _world_update_span = span!(Level::INFO, "world_update").entered();

                // Update world
                res_manager.update(&render_instance);
            }


            // ====== Render ======
            {
                let _render_span = span!(Level::INFO, "render").entered();

                // Handle render event
                let mut should_close = false;
                let render_texture: Option<wde_wgpu::RenderTexture> = match RenderInstance::get_current_texture(&render_instance) {
                    // Redraw to render texture
                    RenderEvent::Redraw(render_texture) => Some(render_texture),
                    // Exit engine
                    RenderEvent::Close => {
                        info!("Closing engine.");
                        should_close = true;
                        None
                    },
                    // Resize window
                    RenderEvent::Resize(width, height) => {
                        debug!("Resizing window to {}x{}.", width, height);
                        None
                    },
                    // No event
                    RenderEvent::None => None,
                };
                if should_close { break; }


                // Render to texture
                if render_texture.is_some() {
                    debug!("Rendering to texture.");

                    // Create command buffer
                    let mut command_buffer = CommandBuffer::new(
                            &render_instance, "Main render").await;
                    
                    {
                        // Create render pass
                        let mut render_pass = command_buffer.create_render_pass(
                            "Main render",
                            &render_texture.as_ref().unwrap().view,
                            Some(Operations {
                                load: LoadOp::Clear(Color { r : 0.1, g: 0.105, b: 0.11, a: 1.0 }),
                                store: StoreOp::Store,
                            }),
                            None);

                        // Render cube
                        info!("Rendering cube.");
                        match res_manager.get::<ModelResource>(&world.get_component::<RenderComponentDynamic>(cube).unwrap().model) {
                            Some(m) => {
                                // Set uniform and storage buffers
                                render_pass.set_bind_group(0, &camera_buffer_bind_group);
                                render_pass.set_bind_group(1, &model_buffer_bind_group);

                                // Set model buffers
                                render_pass.set_vertex_buffer(0, &m.data.as_ref().unwrap().vertex_buffer);
                                render_pass.set_index_buffer(&m.data.as_ref().unwrap().index_buffer);

                                // Set render pipeline
                                match render_pass.set_pipeline(&render_pipeline) {
                                    Ok(_) => {
                                        let _ = render_pass.draw_indexed(0..m.data.as_ref().unwrap().index_count, 0);
                                    },
                                    Err(_) => {}
                                }
                            }
                            None => {},
                        };

                        // Render big model
                        info!("Rendering big model.");
                        match res_manager.get::<ModelResource>(&world.get_component::<RenderComponentDynamic>(big_model).unwrap().model) {
                            Some(m) => {
                                // Set uniform and storage buffers
                                render_pass.set_bind_group(0, &camera_buffer_bind_group);
                                render_pass.set_bind_group(1, &model_buffer_bind_group);

                                // Set model buffers
                                render_pass.set_vertex_buffer(0, &m.data.as_ref().unwrap().vertex_buffer);
                                render_pass.set_index_buffer(&m.data.as_ref().unwrap().index_buffer);

                                // Set render pipeline
                                match render_pass.set_pipeline(&render_pipeline) {
                                    Ok(_) => {
                                        let _ = render_pass.draw_indexed(0..m.data.as_ref().unwrap().index_count, 0);
                                    },
                                    Err(_) => {}
                                }
                            }
                            None => {},
                        };
                    }

                    // Submit command buffer
                    command_buffer.submit(&render_instance);

                    // Present frame
                    let _ = render_instance.present(render_texture.unwrap());
                }
            }

            // Clear the receiver channel
            {
                let _clear_receiver_span = span!(Level::INFO, "clear_receiver").entered();
                while let Ok(_) = event_r.try_recv() {}
            }
        }

        // End
        debug!("\n\n\n======== Ending engine ========");

        // Join window thread
        info!("Joining window thread.");
        {
            let _window_join_span = span!(Level::INFO, "window_join").entered();
            window_join.join().unwrap();
        }
        
        App {}
    }
}