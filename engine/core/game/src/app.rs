use wde_editor_interactions::EditorHandler;
use wde_logger::{error, info};
use wde_window::{Window, LoopEvent};

pub struct App {}

impl App {
    pub fn new() -> Self {
        // Configure environment if on debug mode
        if cfg!(debug_assertions) {
            // Set the RUST_BACKTRACE environment variable to 1
            std::env::set_var("RUST_BACKTRACE", "0");
        }

        // Start editor handler if on debug mode
        let mut editor_handler = None;
        if cfg!(debug_assertions) {
            let h = EditorHandler::new();
            if !h.started() {
                error!("Editor handler failed to start.");
            }
            else {
                editor_handler = Some(h);
            }
        }

        // Create window
        let mut window = Window::new(800, 600, "WaterDropEngine");
        window.run(move |event| {
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
                        // Process editor messages
                        editor_handler.as_mut().unwrap().process();

                        // Set last frame
                        let r = rand::random::<u32>();
                        editor_handler.as_mut().unwrap().set_current_frame(format!("Hello {} world", r).as_bytes());
                    }
                },
            }
        });
        
        App {}
    }
}
