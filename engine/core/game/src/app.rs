use wde_editor_interactions::EditorHandler;
use wde_logger::error;

pub struct App {}

impl App {
    pub fn new() -> Self {
        // Configure environment if on debug mode
        if cfg!(debug_assertions) {
            // Set the RUST_BACKTRACE environment variable to 1
            std::env::set_var("RUST_BACKTRACE", "0");
        }

        // Start editor handler if on debug mode
        if cfg!(debug_assertions) {
            let mut editor_handler = EditorHandler::new();
            if !editor_handler.started() {
                error!("Editor handler failed to start.");
            }
            else {
                loop {
                    // Process editor messages
                    editor_handler.process();

                    // Set last frame
                    let r = rand::random::<u32>();
                    editor_handler.set_current_frame(format!("Hello {} world", r).as_bytes());

                    // Sleep for 1 second
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        }
        
        App {}
    }
}
