//! Define a command buffer to record commands for the GPU.
use bevy::{log::Level, utils::tracing::event};
use wgpu::Texture;

use crate::{buffer::WBuffer, compute_pass::WComputePass, instance::WRenderInstanceData, texture::TextureView};

use super::render_pass::WRenderPass;

/// Type of a color.
pub type Color = wgpu::Color;

/// Type of a load operation.
pub type LoadOp<V> = wgpu::LoadOp<V>;

/// Type of a store operation.
pub type StoreOp = wgpu::StoreOp;

/// Load and store operations for the color texture.
#[derive(Clone, Copy, Debug)]
pub struct Operations<V> {
    pub load: LoadOp<V>,
    pub store: StoreOp,
}

/// Create a new command buffer to record commands for the GPU.
/// The command buffer can be used to create render passes and compute passes.
/// Then, the command buffer can be submitted to the GPU.
/// 
/// # Example
/// 
/// ```
/// let mut command_buffer = WCommandBuffer::new(&instance, "Command Buffer");
/// 
/// {
///     // Create passes
///     let mut render_pass = command_buffer.create_render_pass(
///         "Render Pass", &color_texture,
///         Some(Operations { load: LoadOp::Clear(Color::BLACK), store: StoreOp::Store }),
///         Some(&depth_texture));
///     let mut compute_pass = command_buffer.create_compute_pass("Compute Pass");
/// 
///     // Use the render pass
///     // ...
/// }
/// 
/// // Submit the command buffer to the GPU
/// command_buffer.submit(&instance);
/// ```
pub struct WCommandBuffer {
    pub label: String,
    encoder: wgpu::CommandEncoder,
}

impl std::fmt::Debug for WCommandBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandBuffer")
            .field("label", &self.label)
            .finish()
    }
}

impl WCommandBuffer {
    /// Create a new command buffer.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `label` - The label of the command buffer.
    pub fn new(instance: &WRenderInstanceData<'_>, label: &str) -> Self {
        event!(Level::TRACE, "Creating a command buffer {}.", label);

        // Create command encoder
        let command_encoder = instance.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(format!("{}-command-encoder", label).as_str()),
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
    /// * `depth_texture` - The depth texture to render to if the pipeline has a depth stencil.
    pub fn create_render_pass<'pass>(&'pass mut self, label: &str,
        color_texture: &'pass TextureView,
        color_operations: Option<Operations<Color>>,
        depth_texture: Option<&'pass TextureView>) -> WRenderPass<'pass> {
        event!(Level::TRACE, "Creating a render pass {}.", label);

        let mut depth_attachment = None;
        if depth_texture.is_some() {
            depth_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_texture.unwrap(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            });
        }

        let mut wgpu_color_operations = None;
        if color_operations.as_ref().is_some() {
            wgpu_color_operations = Some(wgpu::Operations {
                load: match color_operations.as_ref().unwrap().load {
                    LoadOp::Clear(color) => wgpu::LoadOp::Clear(color),
                    LoadOp::Load => wgpu::LoadOp::Load,
                },
                store: match color_operations.unwrap().store {
                    StoreOp::Discard => wgpu::StoreOp::Discard,
                    StoreOp::Store => wgpu::StoreOp::Store,
                },
            });
        }

        let render_pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(format!("{}-render-pass", label).as_str()),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_texture,
                resolve_target: None,
                ops: wgpu_color_operations.unwrap_or(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::BLACK),
                    store: wgpu::StoreOp::Store
                }),
            })],
            depth_stencil_attachment: depth_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        WRenderPass::new(label, render_pass)
    }

    /// Create a new compute pass.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the compute pass.
    pub fn create_compute_pass<'pass>(&'pass mut self, label: &str) -> WComputePass<'pass> {
        event!(Level::TRACE, "Creating a compute pass {}.", label);
        let compute_pass = self.encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(format!("{}-compute-pass", label).as_str()),
            timestamp_writes: None
        });

        WComputePass::new(label, compute_pass)
    }

    /// Finish and submit a command buffer.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    pub fn submit(self, instance: &WRenderInstanceData) {
        event!(Level::TRACE, "Submitted command buffer {}.", self.label);
        instance.queue.submit(std::iter::once(self.encoder.finish()));
    }


    /// Copy a buffer to another buffer.
    /// Please use the `copy_from_buffer` method of the buffer to copy data.
    /// 
    /// # Arguments
    /// 
    /// * `source` - The source buffer.
    /// * `destination` - The destination buffer.
    pub fn copy_buffer_to_buffer(&mut self, source: &WBuffer, destination: &WBuffer) {
        event!(Level::TRACE, "Copying buffer {} to buffer {}.", source.label, destination.label);

        self.encoder.copy_buffer_to_buffer(
            &source.buffer, 0,
            &destination.buffer, 0,
            source.buffer.size());
    }

    /// Copy a texture to a buffer.
    /// Please use the `copy_from_texture` method of the buffer to copy data.
    /// 
    /// # Arguments
    /// 
    /// * `source` - The source texture.
    /// * `destination` - The destination buffer.
    /// * `size` - The size of the texture.
    pub fn copy_texture_to_buffer(&mut self, source: &Texture, destination: &WBuffer, size: wgpu::Extent3d) {
        event!(Level::TRACE, "Copying texture to buffer {}.", destination.label);

        // Create texture copy
        let texture_copy = source.as_image_copy();

        // Create buffer copy
        let buffer_copy = wgpu::ImageCopyBuffer {
            buffer: &destination.buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * 4 * size.width),
                rows_per_image: None,
            }
        };

        // Copy texture to buffer
        self.encoder.copy_texture_to_buffer(
            texture_copy,
            buffer_copy,
            size);
    }

    /// Get the encoder of the command buffer.
    /// 
    /// # Returns
    /// 
    /// The encoder of the command buffer.
    pub fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }
}
