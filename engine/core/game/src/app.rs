use tokio::sync::mpsc;
use wde_logger::{info, throw, trace, debug};
use wde_editor_interactions::EditorHandler;
use wde_resources::{ResourcesManager, ModelResource};
use wde_wgpu::{LoopEvent, Window, RenderInstance, RenderEvent, CommandBuffer, LoadOp, Operations, StoreOp, Color, RenderPipeline, Event, WindowEvent};

pub struct App {}

impl App {
    pub async fn new() -> Self {
        // Configure environment if on debug mode
        if cfg!(debug_assertions) {
            // Set the RUST_BACKTRACE environment variable to 1
            std::env::set_var("RUST_BACKTRACE", "0");
        }


        // Create window
        let mut window = Window::new(800, 600, "WaterDropEngine");

        // Create channel to send the window to the runtime
        let (window_t, mut window_r) = mpsc::unbounded_channel();

        // Run window
        trace!("Starting window.");
        let (event_t, mut event_r) = mpsc::unbounded_channel();
        let window_join = std::thread::spawn(move || {
            // Create event loop
            let event_loop = window.create();
            let window_index = window.run(&event_loop);

            // Send window to another channel
            window_t.send(window).unwrap();

            // Run event loop
            event_loop.run(move |event, elwt| {
                match event {
                    // Close window when the close button is pressed
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } if window_id == window_index => {
                        info!("Closing window.");
                        event_t.send(LoopEvent::Close).unwrap_or_else(|e| {
                            throw!("Failed to send close event : {}", e);
                        });
                        elwt.exit();
                    },

                    // Resize window when requested
                    Event::WindowEvent {
                        event: WindowEvent::Resized(size),
                        window_id,
                    } if window_id == window_index => {
                        event_t.send(LoopEvent::Resize(size.width, size.height)).unwrap_or_else(|e| {
                            throw!("Failed to send resize event : {}", e);
                        });
                    },

                    // Ask for redraw when all events are processed
                    Event::AboutToWait => {
                        event_t.send(LoopEvent::Redraw).unwrap_or_else(|e| {
                            throw!("Failed to send redraw event : {}", e);
                        });
                    },

                    // Redraw window when requested
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => {
                        event_t.send(LoopEvent::Redraw).unwrap_or_else(|e| {
                            throw!("Failed to send redraw event : {}", e);
                        });
                    },

                    // Ignore other events
                    _ => ()
                }
            }).unwrap_or_else(|e| {
                throw!("Failed to run event loop : {:?}", e);
            });
        });

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
                    let _ = editor.set_current_frame(format!("Hello {} world", r).as_bytes());
                },
                Err(_) => {}
            }
        }
        
        // Load model
        let handle = res_manager.load::<ModelResource>("models/cube.obj");

        // Create default render pipeline
        let mut render_pipeline = RenderPipeline::new("Main Render");
        let _ = render_pipeline
            .set_shader("
                @fragment
                fn main() -> @location(0) vec4<f32> {
                    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                }
            ", wde_wgpu::ShaderType::Fragment)
            .set_shader("
                @vertex
                fn main() -> @builtin(position) vec4<f32> {
                    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
                }
            ", wde_wgpu::ShaderType::Vertex)
            .init(&render_instance).await;


        loop {
            // Wait for next event
            match event_r.recv().await.unwrap() {
                LoopEvent::Close => { break; },
                LoopEvent::Redraw => { },
                LoopEvent::Resize(_, _) => { continue; },
            }
            trace!("Handling next frame.");

            // Update resources manager
            res_manager.update(&render_instance);

            // Render
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

                    // Set vertex buffer
                    match res_manager.get::<ModelResource>(handle.clone()) {
                        Some(m) => {
                            // Set buffers
                            render_pass.set_vertex_buffer(0, &m.data.as_ref().unwrap().vertex_buffer);
                            render_pass.set_index_buffer(&m.data.as_ref().unwrap().index_buffer);

                            // Set render pipeline
                            match render_pass.set_pipeline(&render_pipeline) {
                                Ok(_) => {
                                    let _ = render_pass.draw_indexed(0..6, 0);
                                },
                                Err(_) => {}
                            }
                        }
                        None => continue,
                    };
                }

                // Submit command buffer
                command_buffer.submit(&render_instance);

                // Present frame
                let _ = render_instance.present(render_texture.unwrap());
            }

            // Clear the receiver channel
            while let Ok(_) = event_r.try_recv() {}
        }

        // Drop resource
        drop(handle);

        // Join window thread
        info!("Joining window thread.");
        window_join.join().unwrap();

        // Dropping modules
        info!("Dropping modules.");
        drop(res_manager);
        drop(window);
        drop(render_instance);
        
        App {}
    }
}
