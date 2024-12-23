use bevy::prelude::*;
use wde_wgpu::render_pipeline::WShaderStages;

use crate::assets::{Material, MaterialBuilder, Texture};

#[derive(Asset, Clone, TypePath)]
/// Describes a terrain rendering material for chunks.
pub struct TerrainChunkMaterial {
    /// The index of the chunk.
    pub chunk_id: (i32, i32),

    /// Albedo color of the material.
    pub albedo: Option<Handle<Texture>>,
}

impl Material for TerrainChunkMaterial {
    fn describe(&self, builder: &mut MaterialBuilder) {
        // Build the material
        builder.add_texture_view(    1, WShaderStages::FRAGMENT, self.albedo.clone());
        builder.add_texture_sampler( 2, WShaderStages::FRAGMENT, self.albedo.clone());
    }

    fn label(&self) -> String {
        format!("terrain-chunk-material-{}-{}", self.chunk_id.0, self.chunk_id.1)
    }
}
