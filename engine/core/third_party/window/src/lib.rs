use wde_logger::info;
use wde_math::Pair2u;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub enum LoopEvent {
    Close,
    Resize(u32, u32),
    Redraw,
}

pub struct Window {
    pub title: String,
    pub size: Pair2u,
    pub window: Option<winit::window::Window>,
}

impl Window {
    /// Create a new os window.
    /// 
    /// # Arguments
    /// 
    /// * `width` - Width of the window.
    /// * `height` - Height of the window.
    /// * `title` - Title of the window.
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        info!("Creating window {}x{}.", width, height);

        Self {
            title: title.to_string(),
            size: (width, height),
            window: None,
        }
    }

    /// Main loop of the window.
    /// 
    /// # Arguments
    /// 
    /// * `callback` - Callback function that is called when an event is received.
    pub fn run<F>(&mut self, mut callback: F) where F: FnMut(LoopEvent) + 'static {
        info!("Starting window main loop.");

        // Create event loop and window
        let event_loop = EventLoop::new().unwrap();
        self.window = Some(WindowBuilder::new()
            .with_title(self.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(self.size.0, self.size.1))
            .with_min_inner_size(winit::dpi::LogicalSize::new(1, 1))
            .with_resizable(true)
            .with_visible(true)
            .with_decorations(true)
            .build(&event_loop).unwrap());
        let window_index = self.window.as_ref().unwrap().id();
        event_loop.set_control_flow(ControlFlow::Poll); // Poll events even when the application is not focused

        // Main loop
        event_loop.run(move |event, elwt| {
            match event {
                // Close window when the close button is pressed
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window_index => {
                    callback(LoopEvent::Close);
                    elwt.exit();
                },

                // Resize window when requested
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    window_id,
                } if window_id == window_index => {
                    callback(LoopEvent::Resize(size.width, size.height));
                    self.size = (size.width, size.height);
                },

                // Ask for redraw when all events are processed
                Event::AboutToWait => {
                    self.window.as_ref().unwrap().request_redraw();
                },

                // Redraw window when requested
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    callback(LoopEvent::Redraw);
                },

                // Ignore other events
                _ => ()
            }
        }).unwrap();

    }


    /// Should be called when the window is resized.
    /// 
    /// # Arguments
    /// 
    /// * `width` - New width of the window.
    /// * `height` - New height of the window.
    pub fn resize(&mut self, width: u32, height: u32) {
        info!("Resizing window {}x{}.", width, height);
        
        self.size = (width, height);
    }
}