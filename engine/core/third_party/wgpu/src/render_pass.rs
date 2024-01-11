use std::ops::Range;

use wde_logger::error;

use crate::{BindGroup, ShaderType, Buffer};

use super::render_pipeline::RenderPipeline;

/// Create a render pass instance.
/// 
/// # Example
/// 
/// ```
/// let mut render_pass = RenderPass::new(...);
/// 
/// // Set render pass dependencies
/// render_pass
///     .set_scissor_rect(0, 0, 800, 600)
///     .set_vertex_buffer([...])
///     .set_index_buffer([...]);
/// 
/// render_pass
///     .set_pipeline(&[...])
///     .set_push_constants(ShaderType::Vertex, &[...])
///     .set_bind_group(0, &[...]);
/// 
/// // Render
/// render_pass.draw_indexed(0..6, 0);
/// ```
pub struct RenderPass<'a> {
    pub label: String,
    render_pass: wgpu::RenderPass<'a>,
}

impl<'a> RenderPass<'a> {
    /// Create a new render pass.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the render pass.
    /// * `render_pass` - The render pass to create.
    pub fn new(label: &str, render_pass: wgpu::RenderPass<'a>) -> Self {
        Self {
            label: label.to_string(),
            render_pass,
        }
    }

    /// Set the pipeline of the render pass.
    /// The bind groups of the pipeline are also set.
    /// 
    /// # Arguments
    /// 
    /// * `pipeline` - The pipeline to set.
    pub fn set_pipeline(&mut self, pipeline: &'a RenderPipeline) -> &mut Self {
        if pipeline.get_pipeline().is_none() {
            error!("Pipeline '{}' is not created yet!", pipeline.label);
        }

        // Set pipeline
        self.render_pass.set_pipeline(&pipeline.get_pipeline().as_ref().unwrap());
        self
    }

    /// Set a vertex buffer of the render pass.
    /// 
    /// # Arguments
    /// 
    /// * `binding` - The binding of the vertex buffer.
    /// * `buffer` - The buffer to set.
    pub fn set_vertex_buffer(&mut self, binding: u32, buffer: &'a Buffer) -> &mut Self {
        self.render_pass.set_vertex_buffer(binding, buffer.buffer.slice(..));
        self
    }

    /// Set the index buffer of the render pass.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - The buffer to set.
    pub fn set_index_buffer(&mut self, buffer: &'a Buffer) -> &mut Self {
        self.render_pass.set_index_buffer(buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);
        self
    }

    /// Set the scissor rect of the render pass.
    /// 
    /// # Arguments
    /// 
    /// * `x` - X coordinate of the scissor rect.
    /// * `y` - Y coordinate of the scissor rect.
    /// * `width` - Width of the scissor rect.
    /// * `height` - Height of the scissor rect.
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) -> &mut Self {
        self.render_pass.set_scissor_rect(x, y, width, height);
        self
    }



    /// Set push constants of the render pass.
    /// 
    /// # Arguments
    /// 
    /// * `types` - The shader types to set the push constants for.
    /// * `data` - The data to set.
    pub fn set_push_constants(&mut self, types: ShaderType, data: &[u8]) -> &mut Self {
        self.render_pass.set_push_constants(match types {
            ShaderType::Vertex => wgpu::ShaderStages::VERTEX,
            ShaderType::Fragment => wgpu::ShaderStages::FRAGMENT,
        }, 0, data);
        self
    }

    /// Set a bind group of the render pass at a binding.
    /// 
    /// # Arguments
    /// 
    /// * `binding` - The binding of the bind group.
    /// * `bind_group` - The bind group to set.
    pub fn set_bind_group(&mut self, binding: u32, bind_group: &'a BindGroup) -> &mut Self {
        self.render_pass.set_bind_group(binding, &bind_group.get_group(), &[]);
        self
    }



    /// Draws primitives from the active vertex buffers.
    /// 
    /// # Arguments
    /// 
    /// * `vertices` - Range of vertices to draw.
    /// * `instances` - Range of instances to draw.
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.render_pass.draw(vertices, instances);
    }

    /// Draws primitives from the active vertex buffers as indexed triangles.
    /// 
    /// # Arguments
    /// 
    /// * `indices` - Range of indices to draw.
    /// * `instance_index` - Index of the instance to draw.
    pub fn draw_indexed(&mut self, indices: Range<u32>, instance_index: u32) {
        self.render_pass.draw_indexed(indices, 0, instance_index..(instance_index+1));
    }
}
