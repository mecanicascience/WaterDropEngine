use wde_logger::debug;
use wde_math::Pair2u;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    window::WindowBuilder,
};

/// Type of a loop
type EventLoop = winit::event_loop::EventLoop<()>;

/// Type of the window event.
pub enum LoopEvent {
    Close,
    Resize(u32, u32),
    Redraw,
}

/// Window handler.
/// 
/// # Example
/// 
/// ```
/// let mut window = Window::new(800, 600, "WaterDropEngine");
/// 
/// // Create window
/// let event_loop = window.create();
/// 
/// // Run window
/// window.run(event_loop, move |event| {
///     match event {
///         // Close window
///         LoopEvent::Close => { ... },
///         // Resize window
///         LoopEvent::Resize(width, height) => { ... },
///         // Redraw window
///         LoopEvent::Redraw => { ... },
///     }
/// });
/// ```
pub struct Window {
    pub title: String,
    pub size: Pair2u,
    pub window: Option<winit::window::Window>
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
        debug!("Creating window {}x{}.", width, height);

        Self {
            title: title.to_string(),
            size: (width, height),
            window: None
        }
    }

    /// Create the window.
    pub fn create(&mut self) -> EventLoop {
        debug!("Creating window.");

        // Create event loop and window
        let event_loop = EventLoop::new().unwrap();
        let window = Some(WindowBuilder::new()
            .with_title(self.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(self.size.0, self.size.1))
            .with_min_inner_size(winit::dpi::LogicalSize::new(1, 1))
            .with_resizable(true)
            .with_visible(true)
            .with_decorations(true)
            .build(&event_loop).unwrap());

        // Set window
        self.window = window;

        // Return event loop
        event_loop
    }

    /// Main loop of the window.
    /// 
    /// # Arguments
    /// 
    /// * `callback` - Callback function that is called when an event is received.
    pub fn run<F>(&mut self, event_loop: EventLoop, mut callback: F) where F: FnMut(LoopEvent) + 'static {
        debug!("Starting window main loop.");
        
        // Get window index
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
}