use tokio::sync::mpsc;
use wde_logger::{info, throw, trace};
use wde_editor_interactions::EditorHandler;
use wde_resources::{ResourcesManager, ModelResource};
use wde_wgpu::{LoopEvent, Window, RenderInstance, RenderEvent, CommandBuffer, LoadOp, Operations, StoreOp, Color, RenderPipeline};

pub struct App {}

impl App {
    pub fn new() -> Self {
        // Configure environment if on debug mode
        if cfg!(debug_assertions) {
            // Set the RUST_BACKTRACE environment variable to 1
            std::env::set_var("RUST_BACKTRACE", "0");
        }

        // Create runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap_or_else(|e| {
                throw!("Failed to create runtime : {:?}", e);
            });


        // Create window
        let mut window = Window::new(800, 600, "WaterDropEngine");
        let event_loop = window.create();

        // Create editor handler
        let mut editor_handler: Option<EditorHandler> = if cfg!(debug_assertions) != true { None } else {
            match EditorHandler::new() {
                Ok(h) => if h.started() { Some(h) } else { None }
                Err(_) => None
            }
        };

        // Create resource manager
        let mut res_manager = ResourcesManager::new();
        
        // Create render instance
        let mut render_instance = runtime.block_on(async {
            RenderInstance::new("WaterDropEngine", Some(&window)).await
        });
        
        
        // Spawn runtime
        let (event_t, mut event_r) = mpsc::unbounded_channel();
        runtime.block_on(async {
            // Load dummy resource
            {
                let handle = res_manager.load::<ModelResource>("test");
                let _ = res_manager.get::<ModelResource>(handle.clone());
            }

            // Handle game loop
            runtime.spawn(async move {
                // Wait for first event
                trace!("Waiting for first event.");
                let _ = event_r.recv().await.unwrap();

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
                    // Wait for render event
                    let _ = event_r.recv().await.unwrap();
                    println!("Render event {:?}", std::thread::current().id());

                    // Handle render event
                    match render_instance.get_current_texture() {
                        // Redraw to render texture
                        RenderEvent::Redraw(render_texture) => {
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

                                // Set render pipeline
                                match render_pass.set_pipeline(&render_pipeline) {
                                    Ok(_) => {
                                        let _ = render_pass.draw_indexed(0..6, 0);
                                    },
                                    Err(_) => {}
                                }
                            }

                            // Submit command buffer
                            command_buffer.submit(&render_instance);

                            // Present frame
                            let _ = render_instance.present(render_texture);
                        },
                        // Exit engine
                        RenderEvent::Close => {
                            info!("Closing engine.");
                            return;
                        },
                        // Resize window
                        RenderEvent::Resize(width, height) => {
                            info!("Resizing window to {}x{}.", width, height);
                        },
                        // No event
                        RenderEvent::None => {},
                    }
                }
            });
        });

        // Run window
        trace!("Running window.");
        window.run(event_loop, move |event| {
            // Handle window events
            match event {
                // Close window when the close button is pressed
                LoopEvent::Close => {
                    info!("Closing engine.");
                    return;
                },
                // Resize window when the window is resized
                LoopEvent::Resize(_, _) => { },
                // Redraw window when the window is redrawn
                LoopEvent::Redraw => {
                    // Send process event
                    event_t.send(event).unwrap();

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
                }
            }
        });
        
        App {}
    }
}
