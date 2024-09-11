use bevy::prelude::*;
use wde_wgpu::{bind_group::WBufferBindingType, render_pipeline::WShaderStages};

use crate::assets::{Material, MaterialBuilder, Mesh, Texture};

#[derive(Asset, Clone, TypePath)]
/// Describes a physically based rendering material.
pub struct PbrMaterial {
    /// The label of the material instance.
    pub label: String,
    /// The albedo color of the material instance.
    pub albedo: (f32, f32, f32),
    /// The texture of the material instance. If none, a dummy texture is used. If some, the texture is used and replaces the albedo color.
    pub texture: Option<Handle<Texture>>,
    /// Metalness of the material instance.
    pub metallic: f32,
    /// Roughness of the material instance.
    pub roughness: f32,
    /// Reflectance of the material instance.
    pub reflectance: f32,
}
impl Default for PbrMaterial {
    fn default() -> Self {
        PbrMaterial {
            label: "pbr-material".to_string(),
            albedo: (1.0, 1.0, 1.0),
            texture: None,
            metallic: 0.0,
            roughness: 0.0,
            reflectance: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct PbrMaterialUniform {
    /// RGB color of the material.
    pub color: [f32; 4],
    /// Whether the material has a texture (1.0) or not (0.0).
    pub has_texture: f32,
    /// Metalness of the material.
    pub metallic: f32,
    /// Roughness of the material.
    pub roughness: f32,
    /// Reflectance of the material.
    pub reflectance: f32,
}
impl Material for PbrMaterial {
    fn describe(&self, builder: &mut MaterialBuilder) {
        // Create the uniform buffer
        let uniform = PbrMaterialUniform {
            color: [self.albedo.0, self.albedo.1, self.albedo.2, 1.0],
            has_texture: if self.texture.is_some() { 1.0 } else { 0.0 },
            metallic: self.metallic,
            roughness: self.roughness,
            reflectance: self.reflectance
        };

        // Build the material
        builder.add_buffer(
            0, WShaderStages::FRAGMENT, WBufferBindingType::Uniform,
            size_of::<PbrMaterialUniform>(), Some(bytemuck::cast_slice(&[uniform]).to_vec()));
        builder.add_texture_view(1, WShaderStages::FRAGMENT, self.texture.clone());
        builder.add_texture_sampler( 2, WShaderStages::FRAGMENT, self.texture.clone());
    }
    fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Bundle)]
/// A bundle of components for a physically based rendering entity.
pub struct PbrBundle {
    pub transform: Transform,
    pub mesh: Handle<Mesh>,
    pub material: Handle<PbrMaterial>,
}
