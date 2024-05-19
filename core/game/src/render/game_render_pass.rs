use wde_resources::ResourcesManager;
use wde_wgpu::{BindGroup, CommandBuffer, RenderInstance, RenderTexture, Texture};

use crate::Scene;

/// A trait that describes a game render pass.
pub trait GameRenderPass {
    /// Render the renderer pass.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance.
    /// * `command_buffer` - The command buffer.
    /// * `render_texture` - The render texture.
    /// * `depth_texture` - The depth texture.
    /// * `camera_buffer_bg` - The camera buffer bind group.
    /// * `scene` - The scene.
    /// * `res_manager` - The resources manager.
    fn render(&mut self, render_instance: &RenderInstance, command_buffer: &mut CommandBuffer, render_texture: &RenderTexture, depth_texture: &Texture, camera_buffer_bg: &BindGroup, scene: &Scene, res_manager: &mut ResourcesManager);

    /// Get the label of the render pass.
    fn label(&self) -> &str;
}
