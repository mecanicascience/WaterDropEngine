use wde_logger::{trace, debug};

use crate::{RenderInstance, Buffer};

use super::render_pass::RenderPass;

/// Create a command buffer.
/// 
/// # Example
/// 
/// ```
/// let mut command_buffer = CommandBuffer::new(&instance, "Command Buffer");
/// 
/// // Create a render pass
/// let mut render_pass = command_buffer.create_render_pass("Render Pass", &color_texture, None, None);
/// 
/// // Set render pass dependencies
/// (...)
/// 
/// // Render
/// render_pass.draw_indexed(0..6, 0);
/// 
/// // Submit the command buffer
/// command_buffer.submit(&instance);
/// ```
pub struct CommandBuffer {
    pub label: String,
    encoder: wgpu::CommandEncoder,
}

impl CommandBuffer {
    /// Create a new command buffer.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `label` - The label of the command buffer.
    pub fn new(instance: &RenderInstance, label: &str) -> Self {
        debug!("Creating command buffer '{}'", label);

        // Create command encoder
        let command_encoder = instance.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(format!("'{}' Command Encoder", label).as_str()),
        });

        Self {
            label: label.to_string(),
            encoder: command_encoder,
        }
    }

    /// Create a new render pass.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the render pass.
    /// * `color_texture` - The color texture to render to.
    /// * `color_operations` - The color operations. If `None`, clear the color texture to black.
    /// * `depth_texture` - The depth texture to render to.
    pub fn create_render_pass<'pass>(&'pass mut self, label: &str,
        color_texture: &'pass wgpu::TextureView,
        color_operations: Option<wgpu::Operations<wgpu::Color>>,
        depth_texture: Option<&'pass wgpu::TextureView>) -> RenderPass<'pass> {
        trace!("Creating render pass '{}'", label);

        let mut depth_attachment = None;
        if depth_texture.is_some() {
            depth_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_texture.unwrap(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            });
        }

        let render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(format!("'{}' Render Pass", label).as_str()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color_texture,
                resolve_target: None,
                ops: color_operations.unwrap_or(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
            })],
            depth_stencil_attachment: depth_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        RenderPass::new(label, render_pass)
    }

    /// Finish and submit a command buffer.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    pub fn submit(self, instance: &RenderInstance) {
        instance.queue.submit(std::iter::once(self.encoder.finish()));
    }


    /// Copy a buffer to another buffer.
    /// 
    /// # Arguments
    /// 
    /// * `source` - The source buffer.
    /// * `destination` - The destination buffer.
    pub fn copy_buffer_to_buffer(&mut self, source: &Buffer, destination: &Buffer) {
        trace!("Copying buffer '{}' to buffer '{}'", source.label, destination.label);

        self.encoder.copy_buffer_to_buffer(
            &source.buffer, 0,
            &destination.buffer, 0,
            source.buffer.size());
    }
}
