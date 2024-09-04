use bevy::{asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext}, ecs::system::lifetimeless::SRes, prelude::*, utils::tracing::error};
use thiserror::Error;
use serde::{Deserialize, Serialize};
use wde_wgpu::instance::WRenderInstance;

use crate::renderer::render_assets::{PrepareAssetError, RenderAsset};


#[derive(Asset, TypePath, Clone)]
pub struct Mesh {
    pub label: String
}

#[derive(Default)]
pub struct MeshLoader;

#[derive(Serialize, Deserialize)]
pub struct MeshLoaderSettings {
    /// The label of the texture.
    pub label: String,
}

impl Default for MeshLoaderSettings {
    fn default() -> Self {
        Self {
            label: "Mesh".to_string()
        }
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
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading mesh on the CPU");

        // Read the texture data
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        Ok(Mesh {
            label: settings.label.clone()
        })
    }

    fn extensions(&self) -> &[&str] {
        &["obj", "fbx"]
    }
}



pub struct GpuMesh {
}
impl RenderAsset for GpuMesh {
    type SourceAsset = Mesh;
    type Param = SRes<WRenderInstance<'static>>;

    fn prepare_asset(
            asset: Self::SourceAsset,
            render_instance: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
        ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        info!("Preparing mesh asset on the GPU");
        
        Ok(GpuMesh {
        })
    }
}
