use wde_logger::{trace, error, debug, throw};
use wgpu::{ShaderStages, BindGroupLayout};

use crate::{RenderInstance, Texture, Vertex, TextureFormat};

/// List of available shaders.
pub enum ShaderType {
    /// Vertex shader.
    Vertex,
    /// Fragment shader.
    Fragment
}

/// List of available topologies.
pub enum Topology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

/// Type of the render pipeline.
pub type RenderPipelineRef = wgpu::RenderPipeline;

/// Type of the pipeline layout.
pub type PipelineLayout = wgpu::PipelineLayout;

// Render pipeline configuration
struct RenderPipelineConfig {
    depth_stencil: bool,
    primitive_topology: wgpu::PrimitiveTopology,
    push_constants: Vec<wgpu::PushConstantRange>,
    bind_groups: Vec<wgpu::BindGroupLayout>,
    vertex_shader: String,
    fragment_shader: String,
}


/// Stores a render pipeline
/// 
/// # Example
/// 
/// ```
/// let mut pipeline = RenderPipeline::new("...");
/// pipeline
///     .set_shader(include_str!("[...].vert"), ShaderType::Vertex)   // Set the vertex shader
///     .set_shader(include_str!("[...].frag"), ShaderType::Fragment) // Set the fragment shader
///     .set_topology(Topology::LineList)            // Change the primitive topology
///     .set_depth_stencil()                         // Enable depth and stencil
///     .add_push_constant(ShaderType::Vertex, 0, 4) // Add a push constant
///     .add_bind_group([...])                       // Add a bind group
///     .init(&instance);                            // Initialize the pipeline
/// ```
pub struct RenderPipeline {
    pub label: String,
    pipeline: Option<RenderPipelineRef>,
    layout: Option<PipelineLayout>,
    config: RenderPipelineConfig,
}

impl RenderPipeline {
    /// Create a new render pipeline.
    /// By default, the render pipeline does not have a depth or stencil.
    /// By default, the primitive topology is `Topology::TriangleList`.
    /// 
    /// # Arguments
    /// 
    /// * `label` - Label of the render pipeline for debugging.
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            pipeline: None,
            layout: None,
            config: RenderPipelineConfig {
                depth_stencil: false,
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                push_constants: Vec::new(),
                bind_groups: Vec::new(),
                vertex_shader: String::new(),
                fragment_shader: String::new(),
            },
        }
    }

    /// Set a given shader.
    /// 
    /// # Arguments
    /// 
    /// * `shader` - The shader source code.
    /// * `shader_type` - The shader type.
    pub fn set_shader(&mut self, shader: &str, shader_type: ShaderType) -> &mut Self {
        match shader_type {
            ShaderType::Vertex => self.config.vertex_shader = shader.to_string(),
            ShaderType::Fragment => self.config.fragment_shader = shader.to_string(),
        };
        self
    }

    /// Set the primitive topology.
    /// 
    /// # Arguments
    /// 
    /// * `topology` - The primitive topology.
    pub fn set_topology(&mut self, topology: Topology) -> &mut Self {
        self.config.primitive_topology = match topology {
            Topology::PointList => wgpu::PrimitiveTopology::PointList,
            Topology::LineList => wgpu::PrimitiveTopology::LineList,
            Topology::LineStrip => wgpu::PrimitiveTopology::LineStrip,
            Topology::TriangleList => wgpu::PrimitiveTopology::TriangleList,
            Topology::TriangleStrip => wgpu::PrimitiveTopology::TriangleStrip,
        };
        self
    }

    /// Set the render pipeline to use depth and stencil.
    pub fn set_depth_stencil(&mut self) -> &mut Self {
        self.config.depth_stencil = true;
        self
    }

    /// Add a bind group via its layout to the render pipeline.
    /// 
    /// # Arguments
    /// 
    /// * `layout` - The bind group layout.
    pub fn add_bind_group(&mut self, layout: BindGroupLayout) -> &mut Self {
        self.config.bind_groups.push(layout);
        self
    }

    /// Add a push constant to the render pipeline.
    /// 
    /// # Arguments
    /// 
    /// * `stages` - The shader stages.
    /// * `offset` - The offset of the push constant.
    /// * `size` - The size of the push constant.
    pub fn add_push_constant(&mut self, stages: ShaderType, offset: u32, size: u32) {
        self.config.push_constants.push(wgpu::PushConstantRange {
            stages : match stages {
                ShaderType::Vertex => ShaderStages::VERTEX,
                ShaderType::Fragment => ShaderStages::FRAGMENT
            },
            range: offset..offset + size,
        });
    }


    /// Initialize a new render pipeline.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - Render instance.
    pub fn init(&mut self, instance: &RenderInstance) {
        debug!("Creating render pipeline '{}'.", self.label);
        let d = &self.config;
        
        // Load shaders
        trace!("Loading shaders for '{}'.", self.label);
        if d.vertex_shader.is_empty() || d.fragment_shader.is_empty() {
            error!("Render pipeline '{}' does not have a vertex or fragment shader.", self.label);
        }
        let shader_module_vert = instance.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(format!("'{}' Render Pipeline Vertex Shader", self.label).as_str()),
            source: wgpu::ShaderSource::Wgsl(self.config.vertex_shader.clone().into())
        });
        let shader_module_frag = instance.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(format!("'{}' Render Pipeline Fragment Shader", self.label).as_str()),
            source: wgpu::ShaderSource::Wgsl(self.config.fragment_shader.clone().into())
        });

        // Create pipeline layout
        trace!("Creating render pipeline instance '{}'.", self.label);
        let layout: wgpu::PipelineLayout = instance.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("'{}' Render Pipeline Layout", self.label).as_str()),
            bind_group_layouts: &d.bind_groups.iter().collect::<Vec<&wgpu::BindGroupLayout>>(),
            push_constant_ranges: &d.push_constants,
        });

        // Create pipeline
        let pipeline = instance.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(format!("'{}' Render Pipeline", self.label).as_str()),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_module_vert,
                entry_point: "main",
                buffers: &vec![Vertex::describe()]
            },
            fragment: Some(wgpu::FragmentState { // Always write to swapchain format
                module: &shader_module_frag,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: match Texture::SWAPCHAIN_FORMAT {
                        TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
                        TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
                        _ => throw!("Swapchain format is not supported!")
                    },
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: d.primitive_topology,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
                unclipped_depth: false,
            },
            depth_stencil: if d.depth_stencil { Some(wgpu::DepthStencilState {
                format: match Texture::DEPTH_FORMAT {
                    TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
                    _ => throw!("Depth format is not supported!")
                },
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }) } else { None },
            multisample: wgpu::MultisampleState::default(),
            multiview: Default::default(),
        });

        // Set pipeline
        self.pipeline = Some(pipeline);
        self.layout = Some(layout);
    }


    /// Get the render pipeline.
    ///
    /// # Returns
    /// 
    /// * `Option<&RenderPipelineRef>` - The render pipeline.
    pub fn get_pipeline(&self) -> Option<&RenderPipelineRef> {
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
}