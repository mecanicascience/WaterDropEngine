use bevy::{asset::Handle, ecs::prelude::*};
use wde_wgpu::{bind_group::BindGroupLayout, render_pipeline::{WFace, WShaderStages, WTopology}};

use crate::assets::Shader;

/// Describes a push constant that will be available to a shader.
/// Note: the size of the push constant must be a multiple of 4 and must not exceed 128 bytes.
#[derive(Clone)]
pub struct PushConstantDescriptor {
    /// The shader stages that the push constant will be available to.
    pub stages: WShaderStages,
    /// The offset in bytes that the push constant will start at.
    pub offset: u32,
    /// The size in bytes of the push constant (note: this must be a multiple of 4 and must not exceed 128 bytes).
    pub size: u32,
}

#[derive(Resource, Clone)]
/// Describes a render pipeline.
pub struct RenderPipelineDescriptor {
    /// The label of the pipeline for debugging (default: "Render Pipeline").
    pub label: &'static str,
    /// The vertex shader of the pipeline (default: None).
    pub vert: Option<Handle<Shader>>,
    /// The fragment shader of the pipeline (default: None).
    pub frag: Option<Handle<Shader>>,
    /// Whether the pipeline should have a depth/stencil attachment (default: false).
    pub depth_stencil: bool,
    /// The bind group layouts that the pipeline will use.
    pub bind_group_layouts: Vec<BindGroupLayout>,
    /// The push constants that the pipeline will use.
    pub push_constants: Vec<PushConstantDescriptor>,
    /// The primitive topology that the pipeline will use (default: TriangleList).
    pub topology: WTopology,
    /// The culling mode that the pipeline will use (default: Back). None will disable culling.
    pub cull_mode: Option<WFace>,
}

impl Default for RenderPipelineDescriptor {
    fn default() -> Self {
        Self {
            label: "Render Pipeline",
            vert: None,
            frag: None,
            depth_stencil: false,
            bind_group_layouts: vec![],
            push_constants: vec![],
            topology: WTopology::TriangleList,
            cull_mode: Some(WFace::Back),
        }
    }
}

