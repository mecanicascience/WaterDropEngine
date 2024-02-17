mod editor;
mod core;
mod widgets;

#[cfg(feature = "editor")]
pub use core::*;
#[cfg(feature = "editor")]
pub use widgets::*;
#[cfg(feature = "editor")]
pub use editor::Editor;

#[cfg(not(feature = "editor"))]
use wde_wgpu::RenderInstance;
#[cfg(not(feature = "editor"))]
use wde_wgpu::RenderTexture;

#[cfg(not(feature = "editor"))]
pub struct Editor;
#[cfg(not(feature = "editor"))]
impl Editor {
    pub async fn new(_window_size: (u32, u32), _instance: &RenderInstance<'_>) -> Self { Editor {} }
    pub async fn render(&mut self, _instance: &RenderInstance<'_>, _texture: &RenderTexture) {}
    pub fn handle_resize(&mut self, _size: (u32, u32)) {}
    pub fn handle_mouse_event(&mut self, _event: &winit::event::Event<()>) {}
    pub fn handle_input_event(&mut self, _event: &winit::event::WindowEvent) {}
    pub fn captures_event(&self, _event: &winit::event::WindowEvent) -> bool { false }
}

