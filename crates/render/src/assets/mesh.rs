use bevy::{asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext}, ecs::system::lifetimeless::SRes, prelude::*, utils::tracing::error};
use thiserror::Error;
use serde::{Deserialize, Serialize};
use wde_wgpu::{buffer::{BufferUsage, WBuffer}, instance::WRenderInstance, vertex::WVertex};

use super::render_assets::{PrepareAssetError, RenderAsset};


#[derive(Asset, TypePath, Clone)]
pub struct Mesh {
    /// The label of the texture.
    pub label: String,
    /// The list of vertices
    pub vertices: Vec<WVertex>,
    /// The list of indices
    pub indices: Vec<u32>,
}

#[derive(Default)]
pub struct MeshLoader;

#[derive(Serialize, Deserialize)]
pub struct MeshLoaderSettings {
    /// The label of the mesh.
    pub label: String,
}

impl Default for MeshLoaderSettings {
    fn default() -> Self {
        Self { label: "Mesh".to_string() }
    }
}

#[derive(Debug, Error)]
pub enum MeshLoaderError {
    #[error("Could not load mesh: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for MeshLoader {
    type Asset = Mesh;
    type Settings = MeshLoaderSettings;
    type Error = MeshLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a MeshLoaderSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading mesh on the CPU from {}.", load_context.asset_path());

        // Read the texture data
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        Ok(Mesh {
            label: settings.label.clone(),
            vertices: vec![],
            indices: vec![],
        })
    }

    fn extensions(&self) -> &[&str] {
        &["obj", "fbx"]
    }
}



pub struct GpuMesh {
    /// The vertex buffer
    pub vertex_buffer: WBuffer,
    /// The index buffer
    pub index_buffer: WBuffer,
    /// The number of indices
    pub index_count: u32,
}
impl RenderAsset for GpuMesh {
    type SourceAsset = Mesh;
    type Param = SRes<WRenderInstance<'static>>;

    fn prepare_asset(
            asset: Self::SourceAsset,
            render_instance: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
        ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        debug!(asset.label, "Loading mesh on the GPU.");

        // Create vertex buffer
        let render_instance = render_instance.data.lock().unwrap();
        let vertex_buffer = WBuffer::new(
            &render_instance,
            format!("{}-vertex", asset.label).as_str(),
            std::mem::size_of::<WVertex>() * asset.vertices.len(),
            BufferUsage::VERTEX,
            Some(bytemuck::cast_slice(&asset.vertices)));

        // Create index buffer
        let index_buffer = WBuffer::new(
            &render_instance,
            format!("{}-indices", asset.label).as_str(),
            std::mem::size_of::<u32>() * asset.indices.len(),
            BufferUsage::INDEX,
            Some(bytemuck::cast_slice(&asset.indices)));
        
        Ok(GpuMesh {
            vertex_buffer,
            index_buffer,
            index_count: asset.indices.len() as u32,
        })
    }
}
