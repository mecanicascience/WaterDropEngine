use bevy::prelude::*;
use wde_wgpu::{bind_group::WBufferBindingType, render_pipeline::WShaderStages};
use crate::assets::{Material, MaterialBuilder, Mesh};

#[derive(Asset, Clone, TypePath)]
/// Describes a simple gizmo material with a color.
pub struct GizmoMaterial {
    /// The label of the material instance.
    pub label: String,
    /// The color of the material instance.
    pub color: [f32; 4],
}

impl Default for GizmoMaterial {
    fn default() -> Self {
        GizmoMaterial {
            label: "gizmo-material".to_string(),
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GizmoMaterialUniform {
    /// Color of the material.
    pub color: [f32; 4],
}

impl Material for GizmoMaterial {
    fn describe(&self, builder: &mut MaterialBuilder) {
        // Create the uniform buffer
        let uniform = GizmoMaterialUniform {
            color: self.color,
        };

        // Build the material
        builder.add_buffer(
            0, WShaderStages::FRAGMENT, WBufferBindingType::Uniform,
            size_of::<GizmoMaterialUniform>(), Some(bytemuck::cast_slice(&[uniform]).to_vec()));
    }

    fn label(&self) -> String {
        self.label.to_string() + "-material"
    }
}

#[derive(Bundle)]
/// A bundle of components for a gizmo entity.
pub struct GizmoBundle {
    pub transform: Transform,
    pub mesh: Handle<Mesh>,
    pub material: Handle<GizmoMaterial>,
}

