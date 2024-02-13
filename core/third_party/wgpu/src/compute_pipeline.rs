use std::fmt::Formatter;

use tracing::{info, trace, error};
use wgpu::ShaderStages;

use crate::{BindGroup, PipelineLayout, RenderError, RenderInstance};

// Compute pipeline configuration
struct ComputePipelineConfig {
    push_constants: Vec<wgpu::PushConstantRange>,
    bind_groups: Vec<wgpu::BindGroupLayout>,
    shader: String
}

impl std::fmt::Debug for ComputePipelineConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputePipelineConfig")
            .field("push_constants", &self.push_constants)
            .field("bind_groups", &self.bind_groups)
            .finish()
    }
}

pub type ComputePipelineRef = wgpu::ComputePipeline;


/// Stores a compute pipeline
/// 
/// # Example
/// 
/// ```
/// let mut pipeline = ComputePipeline::new("Compute Pipeline");
/// pipeline
///     .set_shader(include_str!("[...].comp"))      // Set the compute shader
///     .add_push_constant(ShaderType::Vertex, 0, 4) // Add a push constant
///     .add_bind_group([...])                       // Add a bind group
///     .init(&instance);                            // Initialize the pipeline
/// ```
pub struct ComputePipeline {
    /// Label for the compute pipeline
    pub label: String,
    /// The compute pipeline
    pub pipeline: Option<wgpu::ComputePipeline>,
    /// The pipeline layout
    pub layout: Option<wgpu::PipelineLayout>,
    /// Whether the compute pipeline has been initialized
    pub is_initialized: bool,
    /// Configuration of the compute pipeline
    config: ComputePipelineConfig,
}

impl std::fmt::Debug for ComputePipeline {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputePipeline")
            .field("label", &self.label)
            .finish()
    }
}

impl ComputePipeline {
    /// Create a new compute pipeline.
    /// 
    /// # Arguments
    /// 
    /// * `label` - Label of the render pipeline for debugging.
    pub fn new(label: &str) -> Self {
        info!(label, "Creating compute pipeline.");

        Self {
            label: label.to_string(),
            pipeline: None,
            layout: None,
            is_initialized: false,
            config: ComputePipelineConfig {
                push_constants: Vec::new(),
                bind_groups: Vec::new(),
                shader: String::new()
            },
        }
    }

    /// Set the compute shader of the pipeline.
    /// 
    /// # Arguments
    /// 
    /// * `shader` - The shader source code.
    pub fn set_shader(&mut self, shader: &str) -> &mut Self {
        self.config.shader = shader.to_string();
        self
    }

    /// Add a bind group via its layout to the compute pipeline.
    /// Note that the order of the bind groups will be the same as the order of the bindings in the shader.
    /// 
    /// # Arguments
    /// 
    /// * `group` - The bind group layout to add to the pipeline.
    pub fn add_bind_group(&mut self, group: BindGroup) -> &mut Self {
        self.config.bind_groups.push(group.layout);
        self
    }

    /// Add a push constant to the compute pipeline.
    /// 
    /// # Arguments
    /// 
    /// * `offset` - The offset of the push constant.
    /// * `size` - The size of the push constant.
    pub fn add_push_constant(&mut self, size: u32) -> &mut Self {
        self.config.push_constants.push(wgpu::PushConstantRange {
            stages : ShaderStages::COMPUTE,
            range: 0..0 + size,
        });
        self
    }

    /// Initialize a new compute pipeline.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - Render instance.
    /// 
    /// # Returns
    /// 
    /// * `Result<(), RenderError>` - The result of the initialization.
    pub fn init(&mut self, instance: &RenderInstance) -> Result<(), RenderError> {
        trace!(self.label, "Initializing compute pipeline.");
        let d = &self.config;

        // Security checks
        if d.shader.is_empty() {
            error!(self.label, "Pipeline does not have a compute shader.");
            return Err(RenderError::MissingShader);
        }

        // Load shaders
        trace!(self.label, "Loading shader.");
        let shader_module = instance.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(format!("'{}' Compute Pipeline Shader", self.label).as_str()),
            source: wgpu::ShaderSource::Wgsl(self.config.shader.clone().into())
        });

        // Create pipeline layout
        trace!(self.label, "Creating compute pipeline instance.");
        let layout = instance.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("'{}' Compute Pipeline Layout", self.label).as_str()),
            bind_group_layouts: &d.bind_groups.iter().collect::<Vec<&wgpu::BindGroupLayout>>(),
            push_constant_ranges: &d.push_constants,
        });

        // Create a compute pipeline
        let pipeline = instance.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(format!("'{}' Compute Pipeline", self.label).as_str()),
            layout: Some(&layout),
            module: &shader_module,
            entry_point: "main",
        });

        // Set pipeline
        self.pipeline = Some(pipeline);
        self.layout = Some(layout);
        self.is_initialized = true;

        Ok(())
    }


    /// Get the compute pipeline.
    ///
    /// # Returns
    /// 
    /// * `Option<&RenderPipelineRef>` - The compute pipeline.
    pub fn get_pipeline(&self) -> Option<&ComputePipelineRef> {
        self.pipeline.as_ref()
    }

    /// Get the pipeline layout.
    /// 
    /// # Returns
    /// 
    /// * `Option<&PipelineLayout>` - The pipeline layout.
    pub fn get_layout(&self) -> Option<&PipelineLayout> {
        self.layout.as_ref()
    }

    /// Check if the compute pipeline is initialized.
    ///
    /// 
    /// # Returns
    /// 
    /// * `bool` - True if the compute pipeline is initialized, false otherwise.
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Drop for ComputePipeline {
    fn drop(&mut self) {
        info!(self.label, "Dropping compute pipeline.");
    }
}
