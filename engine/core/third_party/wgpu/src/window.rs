use wde_logger::{throw, info};
use wde_math::Pair2u;
use winit::{
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder, platform::windows::EventLoopBuilderExtWindows,
};

/// Type of an event
pub type Event = winit::event::Event<()>;
pub type WindowEvent = winit::event::WindowEvent;
pub type WindowIndex = winit::window::WindowId;

/// Type of a loop
type EventLoop = winit::event_loop::EventLoop<()>;

/// Input type
pub type PhysicalKey = winit::keyboard::PhysicalKey;
pub type KeyCode = winit::keyboard::KeyCode;

/// Element state
pub type ElementState = winit::event::ElementState;

/// Type of the window event.
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug)]
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
    #[tracing::instrument]
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        info!(width, height, "Creating new window.");

        Self {
            title: title.to_string(),
            size: (width, height),
            window: None
        }
    }

    /// Create the window.
    /// If the event loop or window fails to be created, the application will panic.
    /// 
    /// # Returns
    /// 
    /// * `event_loop` - The event loop of the window.
    #[tracing::instrument]
    pub fn create(&mut self) -> EventLoop {
        info!("Creating window.");

        // Create event loop and window
        let event_loop = EventLoopBuilder::new().with_any_thread(true).build().unwrap();
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
    /// * `event_loop` - The event loop of the window.
    /// 
    /// # Returns
    /// 
    /// * `window_index` - The index of the window.
    #[tracing::instrument]
    pub fn run(&mut self, event_loop: &EventLoop) -> WindowIndex {
        info!("Starting window main loop.");

        // Check if the window is created
        if self.window.is_none() {
            throw!("Window is not created. Call 'create()' before 'run()'.");
        }
        
        // Get window index
        let window_index = self.window.as_ref().unwrap().id();
        event_loop.set_control_flow(ControlFlow::Poll); // Poll events even when the application is not focused

        // Return window index
        window_index
    }

    /// Resize the window.
    /// 
    /// # Arguments
    /// 
    /// * `width` - The new width of the window.
    /// * `height` - The new height of the window.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
    }
}

impl Drop for Window {
    #[tracing::instrument]
    fn drop(&mut self) {
        info!("Dropping window.");
    }
}
