use std::{any::Any, sync::{Arc, Mutex}};

use tracing::{debug, error, info, trace};
use wde_wgpu::{RenderInstance, Texture};

use crate::{LoadedFlag, Resource, ResourceDescription, ResourceType, ResourcesManager};

/// Temporary data to be transferred.
#[derive(Clone, Debug)]
struct TempTextureData {
    size: (u32, u32),
    format: wde_wgpu::TextureFormat,
    usage: wde_wgpu::TextureUsages,
    data_u8: Option<Vec<u8>>,
    data_f32: Option<Vec<f32>>,
}

/// Resource data.
#[derive(Debug)]
pub struct TextureData {
    /// Texture.
    pub texture: Texture,
}

/// Store a texture resource loaded from a texture file.
/// This resource is loaded asynchronously.
/// The data are stored in the `data` field when loaded.
#[derive(Debug)]
pub struct TextureResource {
    /// Label of the texture.
    pub label: String,
    /// Path of the texture file.
    pub path: String,
    /// Texture data.
    pub data: Option<TextureData>,
    /// Loaded state of the texture.
    loaded: bool,

    // Async loading
    async_loaded: LoadedFlag,
    sync_receiver: std::sync::mpsc::Receiver<TempTextureData>,
}

impl Resource for TextureResource {
    #[tracing::instrument]
    fn new(desc: ResourceDescription) -> Self where Self: Sized {
        info!(desc.label, "Creating texture resource.");

        // Check if resource type is correct
        if desc.resource_type != Self::resource_type() {
            error!(desc.label, "Trying to create a texture resource with a non texture resource description.");
            return Self {
                label: desc.label.to_string(),
                path: desc.source.to_string(),
                data: None,
                loaded: false,
                async_loaded: LoadedFlag { flag: Arc::new(Mutex::new(false)), },
                sync_receiver: std::sync::mpsc::sync_channel(1).1
            };
        }

        // Create sync resources
        let async_loaded = LoadedFlag { flag: Arc::new(Mutex::new(false)), };
        let async_loaded_c = Arc::clone(&async_loaded.flag);
        let (sync_sender, sync_receiver) = std::sync::mpsc::sync_channel(1);
        let path_c = desc.source.to_string();
        
        // Create async task
        let task = async move {
            // File path
            let current_exe = std::env::current_exe().unwrap();
            let path_f_tmp = current_exe.as_path().parent().unwrap();
            #[cfg(target_os = "windows")]
            let path_f = path_f_tmp.join(path_c.clone().replace("/", "\\"));
            #[cfg(target_os = "linux")]
            let path_f = path_f_tmp.join(path_c.clone().replace("\\", "/"));

            // Read texture format and usage
            let tex_format = match desc.data.clone().unwrap_or_default().get("format") {
                Some(format) => {
                    match format.as_str() {
                        Some("R8Unorm") => wde_wgpu::TextureFormat::R8Unorm,
                        Some("R8Snorm") => wde_wgpu::TextureFormat::R8Snorm,
                        Some("R8Uint") => wde_wgpu::TextureFormat::R8Uint,
                        Some("R8Sint") => wde_wgpu::TextureFormat::R8Sint,

                        Some("R16Uint") => wde_wgpu::TextureFormat::R16Uint,
                        Some("R16Sint") => wde_wgpu::TextureFormat::R16Sint,
                        Some("R16Unorm") => wde_wgpu::TextureFormat::R16Unorm,
                        Some("R16Snorm") => wde_wgpu::TextureFormat::R16Snorm,
                        Some("R16Float") => wde_wgpu::TextureFormat::R16Float,
                        Some("Rg8Unorm") => wde_wgpu::TextureFormat::Rg8Unorm,
                        Some("Rg8Snorm") => wde_wgpu::TextureFormat::Rg8Snorm,
                        Some("Rg8Uint") => wde_wgpu::TextureFormat::Rg8Uint,
                        Some("Rg8Sint") => wde_wgpu::TextureFormat::Rg8Sint,

                        Some("R32Uint") => wde_wgpu::TextureFormat::R32Uint,
                        Some("R32Sint") => wde_wgpu::TextureFormat::R32Sint,
                        Some("R32Float") => wde_wgpu::TextureFormat::R32Float,
                        Some("Rg16Uint") => wde_wgpu::TextureFormat::Rg16Uint,
                        Some("Rg16Sint") => wde_wgpu::TextureFormat::Rg16Sint,
                        Some("Rg16Unorm") => wde_wgpu::TextureFormat::Rg16Unorm,
                        Some("Rg16Snorm") => wde_wgpu::TextureFormat::Rg16Snorm,
                        Some("Rg16Float") => wde_wgpu::TextureFormat::Rg16Float,
                        Some("Rgba8Unorm") => wde_wgpu::TextureFormat::Rgba8Unorm,
                        Some("Rgba8UnormSrgb") => wde_wgpu::TextureFormat::Rgba8UnormSrgb,
                        Some("Rgba8Snorm") => wde_wgpu::TextureFormat::Rgba8Snorm,
                        Some("Rgba8Uint") => wde_wgpu::TextureFormat::Rgba8Uint,
                        Some("Rgba8Sint") => wde_wgpu::TextureFormat::Rgba8Sint,
                        Some("Rgb10a2Unorm") => wde_wgpu::TextureFormat::Rgb10a2Unorm,
                        Some("Rg11b10Float") => wde_wgpu::TextureFormat::Rg11b10Float,
                        Some("Rg32Uint") => wde_wgpu::TextureFormat::Rg32Uint,
                        Some("Rg32Sint") => wde_wgpu::TextureFormat::Rg32Sint,
                        Some("Rg32Float") => wde_wgpu::TextureFormat::Rg32Float,
                        Some("Rgba16Uint") => wde_wgpu::TextureFormat::Rgba16Uint,
                        Some("Rgba16Sint") => wde_wgpu::TextureFormat::Rgba16Sint,
                        Some("Rgba16Float") => wde_wgpu::TextureFormat::Rgba16Float,
                        Some("Rgba32Uint") => wde_wgpu::TextureFormat::Rgba32Uint,
                        Some("Rgba32Sint") => wde_wgpu::TextureFormat::Rgba32Sint,
                        Some("Rgba32Float") => wde_wgpu::TextureFormat::Rgba32Float,
                        _ => {
                            error!(path_c, "Failed to read texture format.");
                            wde_wgpu::TextureFormat::Rgba8Unorm
                        }
                    }
                },
                None => {
                    wde_wgpu::TextureFormat::Rgba8Unorm
                }
            };

            let tex_usage = match desc.data.unwrap_or_default().get("usage") {
                Some(usage) => {
                    let mut usage_flag = wde_wgpu::TextureUsages::empty();
                    for u in usage.as_array().unwrap_or(&vec![]).iter() {
                        match u.as_str() {
                            Some("COPY_SRC") => usage_flag |= wde_wgpu::TextureUsages::COPY_SRC,
                            Some("COPY_DST") => usage_flag |= wde_wgpu::TextureUsages::COPY_DST,
                            Some("TEXTURE_BINDING") => usage_flag |= wde_wgpu::TextureUsages::TEXTURE_BINDING,
                            Some("STORAGE_BINDING") => usage_flag |= wde_wgpu::TextureUsages::STORAGE_BINDING,
                            Some("RENDER_ATTACHMENT") => usage_flag |= wde_wgpu::TextureUsages::RENDER_ATTACHMENT,
                            _ => {
                                error!(path_c, "Failed to read texture usage.");
                            }
                        }
                    }
                    usage_flag
                },
                None => {
                    wde_wgpu::TextureUsages::empty()
                }
            };

            // Open file
            trace!(path_c, "Loading texture.");
            let data = match stb_image::image::load_with_depth(&path_f, 4, false) {
                stb_image::image::LoadResult::Error(e) => {
                    error!(path_c, "Failed to load texture : {}.", e);
                    return;
                },
                stb_image::image::LoadResult::ImageU8(image) => {
                    TempTextureData {
                        size: (image.width as u32, image.height as u32),
                        format: tex_format,
                        usage: tex_usage,
                        data_u8: Some(image.data),
                        data_f32: None,
                    }
                }
                stb_image::image::LoadResult::ImageF32(image) => {
                    TempTextureData {
                        size: (image.width as u32, image.height as u32),
                        format: tex_format,
                        usage: tex_usage,
                        data_u8: None,
                        data_f32: Some(image.data),
                    }
                }
            };

            // Set loading to false
            let mut flag = async_loaded_c.lock().unwrap();
            *flag = true;

            // Log that the texture is async loaded
            debug!(path_c, "Texture is async loaded.");

            // Send data
            sync_sender.send(data).unwrap_or_else(|e| {
                error!("Failed to send texture data : {}.", e);
            });
        };
        tokio::task::spawn(task);

        Self {
            label: desc.label.to_string(),
            path: desc.source.to_string(),
            data: None,
            loaded: false,
            async_loaded,
            sync_receiver
        }
    }

    #[tracing::instrument]
    fn sync_load(&mut self, instance: &RenderInstance, _res_manager: &ResourcesManager) {
        // Check if the texture is async loaded
        if !self.async_loaded() {
            error!("Trying to sync load a texture that is not async loaded.");
            return;
        }
        debug!(self.label, "Sync loading texture.");

        // Receive data
        let temp_data = self.sync_receiver.recv().unwrap_or_else(|e| {
            error!("Failed to receive texture data : {}.", e);
            TempTextureData {
                size: (0, 0),
                format: wde_wgpu::TextureFormat::Rgba8Unorm,
                usage: wde_wgpu::TextureUsages::empty(),
                data_u8: None,
                data_f32: None,
            }
        });

        // Create texture
        let texture = Texture::new(
            &instance, &self.label, temp_data.size,
            temp_data.format, temp_data.usage);

        // Write texture data
        if let Some(data_u8) = temp_data.data_u8 {
            texture.copy_from_buffer(&instance, &data_u8);
        } else if let Some(data_f32) = temp_data.data_f32 {
            texture.copy_from_buffer(&instance, &bytemuck::cast_slice(data_f32.as_slice()));
        }

        // Set data
        self.data = Some(TextureData {
            texture
        });

        // Set loaded flag
        self.loaded = true;
    }


    // Inherited methods
    fn async_loaded(&self) -> bool { self.async_loaded.flag.lock().unwrap().clone() }
    fn loaded(&self) -> bool { self.loaded }
    fn resource_type() -> ResourceType { ResourceType::Texture }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn as_any(&self) -> &dyn Any { self }
}

impl Drop for TextureResource {
    #[tracing::instrument]
    fn drop(&mut self) {
        info!(self.label, "Unloading texture resource.");
    }
}
