use wde_resources::ResourcesManager;
use wde_wgpu::{BindGroup, CommandBuffer, RenderTexture, Texture};

use crate::Scene;

/// A trait that describes a game render pass.
pub trait GameRenderPass {
    /// Render the renderer pass.
    /// 
    /// # Arguments
    /// 
    /// * `render_pass` - The render pass
    /// * `scene` - The scene to render
    /// * `res_manager` - The resources manager
    fn render(&self, command_buffer: &mut CommandBuffer, render_texture: &RenderTexture, depth_texture: &Texture, camera_buffer_bg: &BindGroup, scene: &Scene, res_manager: &mut ResourcesManager);

    /// Get the label of the render pass.
    fn label(&self) -> &str;
}
