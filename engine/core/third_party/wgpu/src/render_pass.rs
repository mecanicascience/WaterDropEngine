use std::ops::Range;

use wde_logger::{error, debug};

use crate::{BindGroup, ShaderType, Buffer, RenderError};

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
    pipeline_set: bool,
    vertex_buffer_set: bool,
    index_buffer_set: bool,
}

impl std::fmt::Debug for RenderPass<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderPass")
            .field("label", &self.label)
            .field("pipeline_set", &self.pipeline_set)
            .field("vertex_buffer_set", &self.vertex_buffer_set)
            .field("index_buffer_set", &self.index_buffer_set)
            .finish()
    }
}

impl<'a> RenderPass<'a> {
    /// Create a new render pass.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the render pass.
    /// * `render_pass` - The render pass to create.
    #[tracing::instrument]
    pub fn new(label: &str, render_pass: wgpu::RenderPass<'a>) -> Self {
        debug!(label, "Creating render pass.");

        Self {
            label: label.to_string(),
            render_pass,
            pipeline_set: false,
            vertex_buffer_set: false,
            index_buffer_set: false,
        }
    }

    /// Set the pipeline of the render pass.
    /// The bind groups of the pipeline are also set.
    /// 
    /// # Arguments
    /// 
    /// * `pipeline` - The pipeline to set.
    /// 
    /// # Errors
    /// 
    /// * `RenderError::PipelineNotInitialized` - The pipeline is not initialized.
    pub fn set_pipeline(&mut self, pipeline: &'a RenderPipeline) -> Result<&mut Self, RenderError> {
        if pipeline.get_pipeline().is_none() {
            error!(pipeline.label, "Pipeline is not created yet.");
            return Err(RenderError::PipelineNotInitialized);
        }

        // Set pipeline
        self.render_pass.set_pipeline(&pipeline.get_pipeline().as_ref().unwrap());
        self.pipeline_set = true;
        Ok(self)
    }

    /// Set a vertex buffer of the render pass.
    /// 
    /// # Arguments
    /// 
    /// * `binding` - The binding of the vertex buffer.
    /// * `buffer` - The buffer to set.
    pub fn set_vertex_buffer(&mut self, binding: u32, buffer: &'a Buffer) -> &mut Self {
        self.render_pass.set_vertex_buffer(binding, buffer.buffer.slice(..));
        self.vertex_buffer_set = true;
        self
    }

    /// Set the index buffer of the render pass.
    /// 
    /// # Arguments
    /// 
    /// * `buffer` - The buffer to set.
    pub fn set_index_buffer(&mut self, buffer: &'a Buffer) -> &mut Self {
        self.render_pass.set_index_buffer(buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.index_buffer_set = true;
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
    /// 
    /// # Errors
    /// 
    /// * `RenderError::PipelineNotSet` - The pipeline is not set.
    /// * `RenderError::MissingVertexBuffer` - The vertex buffer is not set.
    #[tracing::instrument]
    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) -> Result<(), RenderError> {
        if !self.pipeline_set {
            error!(self.label, "Pipeline is not set.");
            return Err(RenderError::PipelineNotSet);
        }
        if !self.vertex_buffer_set {
            error!(self.label, "Vertex buffer is not set.");
            return Err(RenderError::MissingVertexBuffer);
        }
        self.render_pass.draw(vertices, instances);
        Ok(())
    }

    /// Draws primitives from the active vertex buffers as indexed triangles.
    /// 
    /// # Arguments
    /// 
    /// * `indices` - Range of indices to draw.
    /// * `instance_index` - Index of the instance to draw. This will use the instance at the index and the next instance.
    /// 
    /// # Errors
    /// 
    /// * `RenderError::PipelineNotSet` - The pipeline is not set.
    /// * `RenderError::MissingVertexBuffer` - The vertex buffer is not set.
    /// * `RenderError::MissingIndexBuffer` - The index buffer is not set.
    #[tracing::instrument]
    pub fn draw_indexed(&mut self, indices: Range<u32>, instance_index: u32) -> Result<(), RenderError> {
        if !self.pipeline_set {
            error!(self.label, "Pipeline is not set.");
            return Err(RenderError::PipelineNotSet);
        }
        if !self.vertex_buffer_set {
            error!(self.label, "Vertex buffer is not set.");
            return Err(RenderError::MissingVertexBuffer);
        }
        if !self.index_buffer_set {
            error!(self.label, "Index buffer is not set.");
            return Err(RenderError::MissingIndexBuffer);
        }
        self.render_pass.draw_indexed(indices, 0, instance_index..(instance_index+1));
        Ok(())
    }
}

impl Drop for RenderPass<'_> {
    #[tracing::instrument]
    fn drop(&mut self) {
        debug!(self.label, "Dropping render pass.");
    }
}
