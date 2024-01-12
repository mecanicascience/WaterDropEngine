use wde_logger::{info, throw};
use wde_editor_interactions::EditorHandler;
use wde_resources::{ResourcesManager, DummyResource};
use wde_wgpu::{LoopEvent, Window, RenderInstance, RenderEvent, CommandBuffer, LoadOp, Operations, StoreOp, Color, RenderPipeline};

pub struct App {}

impl App {
    pub async fn new() -> Self {
        // Configure environment if on debug mode
        if cfg!(debug_assertions) {
            // Set the RUST_BACKTRACE environment variable to 1
            std::env::set_var("RUST_BACKTRACE", "0");
        }

        // Start editor handler if on debug mode
        let mut editor_handler = None;
        if cfg!(debug_assertions) {
            let h: Option<EditorHandler> = match EditorHandler::new() {
                Ok(h) => Some(h),
                Err(_) => None,
            };
            if h.is_some() && h.as_ref().unwrap().started() {
                editor_handler = Some(h);
            }
        }

        // Load resource
        let mut res_manager = ResourcesManager::new();
        {
            let handle = res_manager.load::<DummyResource>("test");
            let _ = res_manager.get::<DummyResource>(handle.clone());
        }

        // Create window
        let mut window = Window::new(800, 600, "WaterDropEngine");
        let event_loop = window.create();

        // Create render instance
        let mut render_instance = RenderInstance::new("WaterDropEngine", Some(&window)).await;

        // Create default render pipeline
        let mut render_pipeline = RenderPipeline::new("Main Render");
        render_pipeline
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
            .init(&render_instance)
            .unwrap_or_else(|e| {
                throw!("Failed to initialize render pipeline : {:?}", e);
            });

        // Run window
        window.run(event_loop, move |event| {
            // Handle window events
            match event {
                // Close window when the close button is pressed
                LoopEvent::Close => {
                    info!("Closing engine.");
                    return;
                },
                // Resize window when the window is resized
                LoopEvent::Resize(width, height) => {
                    info!("Resizing window to {}x{}.", width, height);
                },
                // Redraw window when the window is redrawn
                LoopEvent::Redraw => {
                    info!("Redrawing window.");

                    // Handle editor messages and push new frame
                    if editor_handler.is_some() {
                        let editor = editor_handler.as_mut().unwrap();
                        if editor.is_some() {
                            match editor.as_mut().unwrap().process() {
                                Ok(_) => {
                                    // Set last frame
                                    let r = rand::random::<u32>();
                                    let _ = editor.as_mut().unwrap().set_current_frame(format!("Hello {} world", r).as_bytes());
                                },
                                Err(_) => {}
                            }
                        }
                    }

                    // Handle render event
                    match render_instance.get_current_texture() {
                        // Redraw window
                        RenderEvent::Redraw(render_texture) => {
                            // Create command buffer
                            let mut command_buffer = CommandBuffer::new(&render_instance, "Main render");

                            // Render frame
                            {
                                // Create render pass
                                let mut render_pass = command_buffer.create_render_pass("Main render",
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
                        // Close window
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
                },
            }
        });
        
        App {}
    }
}
