use bevy::{asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext}, ecs::system::lifetimeless::SRes, prelude::*, utils::tracing::error};
use stb_image::image::LoadResult;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use wde_wgpu::instance::WRenderInstance;

use crate::renderer::render_assets::{PrepareAssetError, RenderAsset};

pub type TextureFormat = wde_wgpu::texture::TextureFormat;
pub type TextureUsages = wde_wgpu::texture::TextureUsages;


#[derive(Asset, TypePath, Clone)]
pub struct Texture {
    pub label: String,
    pub size: (usize, usize, usize),
    pub format: TextureFormat,
    pub usages: TextureUsages,
    pub data: Vec<u8>,
    pub is_f32: bool
}

#[derive(Default)]
pub struct TextureLoader;

#[derive(Serialize, Deserialize)]
pub struct TextureLoaderSettings {
    /// The label of the texture.
    pub label: String,
    /// The format of the texture (by default RGBA8Unorm).
    pub format: TextureFormat,
    /// The usages of the texture (by default TEXTURE_BINDING).
    pub usages: TextureUsages,
    /// The depth of the texture (ex: RGB 3, RGBA 4). If None, the depth is 1.
    pub force_depth: Option<usize>
}

impl Default for TextureLoaderSettings {
    fn default() -> Self {
        Self {
            label: "Texture".to_string(),
            format: TextureFormat::Rgba8Unorm,
            usages: TextureUsages::TEXTURE_BINDING,
            force_depth: None
        }
    }
}

#[derive(Debug, Error)]
pub enum TextureLoaderError {
    #[error("Could not load texture: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for TextureLoader {
    type Asset = Texture;
    type Settings = TextureLoaderSettings;
    type Error = TextureLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a TextureLoaderSettings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading texture on the CPU");

        // Read the texture data
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let (data, size, is_f32) = {
            let loaded_image = if let Some(depth) = settings.force_depth {
                stb_image::image::load_from_memory_with_depth(&bytes, depth, false)
            } else {
                stb_image::image::load_from_memory(&bytes)
            };
            match loaded_image {
                LoadResult::Error(e) => {
                    error!("Could not load texture: {}", e);
                    return Err(TextureLoaderError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)));
                },
                LoadResult::ImageU8(image) => (image.data, (image.width, image.height, image.depth), false),
                LoadResult::ImageF32(image) => (bytemuck::cast_slice(image.data.as_slice()).to_owned(), (image.width, image.height, image.depth), true)
            }
        };

        Ok(Texture {
            label: settings.label.clone(),
            format: settings.format,
            usages: settings.usages,
            size,
            data,
            is_f32
        })
    }

    fn extensions(&self) -> &[&str] {
        &["png", "jpg"]
    }
}



pub struct GpuTexture {
    pub _texture: wde_wgpu::texture::WTexture,
}
impl RenderAsset for GpuTexture {
    type SourceAsset = Texture;
    type Param = SRes<WRenderInstance<'static>>;

    fn prepare_asset(
            asset: Self::SourceAsset,
            render_instance: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
        ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        info!("Preparing texture asset on the GPU");

        let render_instance = render_instance.data.as_ref().lock().unwrap();

        // Check if format is compatible with the image depth
        if asset.size.0 as u32 * asset.size.1 as u32 * asset.size.2 as u32 * if asset.is_f32 { 4 } else { 1 } != asset.data.len() as u32 {
            return Err(PrepareAssetError::Fatal(format!("Format of size {:?} (width, height, depth) is not compatible with the image data of size {}", asset.size, asset.data.len())));
        }

        // Create the texture
        let texture = wde_wgpu::texture::WTexture::new(
            &render_instance, &asset.label, (asset.size.0 as u32, asset.size.1 as u32),
            asset.format, asset.usages);

        // Copy the texture data
        texture.copy_from_buffer(&render_instance, &asset.data, asset.size.2 as u32, asset.is_f32);

        Ok(GpuTexture {
            _texture: texture
        })
    }
}
