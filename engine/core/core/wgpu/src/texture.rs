use wde_logger::{debug, trace};

use crate::{RenderInstance, BindGroup};

/// Texture view
pub type TextureView = wgpu::TextureView;

/// Texture usages.
#[derive(Clone, Copy, PartialEq)]
pub enum TextureUsage {
    Sampled,
    Storage,
    OutputAttachment,
}

// Texture format.
#[derive(Clone, Copy, PartialEq)]
pub enum TextureFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Depth32Float,
}

/// Texture struct.
pub struct Texture {
    pub label: String,
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: wgpu::Sampler,
    pub size: (u32, u32),
}

impl Texture {
    /// The swap chain texture format.
    pub const SWAPCHAIN_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;
    /// The depth texture format.
    pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

    /// Create a new texture.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - Game instance.
    /// * `label` - Label of the texture.
    /// * `size` - Size of the texture.
    /// * `format` - Format of the texture.
    /// * `usage` - Usage of the texture.
    pub fn new(instance: &RenderInstance, label: &str, size: (u32, u32), format: TextureFormat, usage: TextureUsage) -> Self {
        debug!("Creating texture '{}'.", label);
        
        // Create texture
        let f = match format {
            TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
            TextureFormat::Bgra8UnormSrgb => wgpu::TextureFormat::Bgra8UnormSrgb,
            TextureFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
        };
        let texture = instance.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(format!("'{}' Texture", label).as_str()),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: f,
            usage: match usage {
                TextureUsage::Sampled => wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                TextureUsage::Storage => wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
                TextureUsage::OutputAttachment => wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            },
            view_formats: &[f]
        });

        // Create texture view
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(format!("'{}' Texture View", label).as_str()),
            format: if format == Self::DEPTH_FORMAT {
                None
            } else {
                Some(f)
            },
            dimension: if format == Self::DEPTH_FORMAT {
                None
            } else {
                Some(wgpu::TextureViewDimension::D2)
            },
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            base_array_layer: 0,
            mip_level_count: None,
            array_layer_count: None
        });

        // If depth texture, set compare sample
        let compare = if format == Self::DEPTH_FORMAT {
            Some(wgpu::CompareFunction::LessEqual)
        } else {
            None
        };

        // Create sampler
        let sampler = instance.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(format!("'{}' Texture Sampler", label).as_str()),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare,
            anisotropy_clamp: 1,
            border_color: None,
        });

        // Return texture
        Self {
            label: label.to_string(),
            texture,
            view,
            sampler,
            size,
        }
    }


    /// Create a texture bind group.
    /// The bindings in group are 0: texture view, 1: sampler.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - Game instance.
    pub fn create_bind_group(&self, instance: &RenderInstance) -> BindGroup {
        trace!("Creating bind group for texture '{}'.", self.label);

        // Create bind group layout
        let layout_entries = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            }
        ];
        let layout = instance.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(format!("Texture '{}' Bind Group Layout", self.label).as_str()),
            entries: &layout_entries,
        });
        
        // Create bind group
        let texture_bind_group = instance.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("'{}' Texture Bind Group", self.label).as_str()),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                }
            ],
        });

        // Return texture bind group
        BindGroup::new(self.label.clone(), texture_bind_group)
    }



    /// Copy buffer to texture.
    /// It is assumed that the buffer is the same size as the texture.
    /// It will be copied on the next queue submit.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - Game instance.
    /// * `buffer` - Image buffer.
    pub fn copy_from_buffer(&self, instance: &RenderInstance, buffer: &[u8]) {
        trace!("Copying buffer to texture '{}'.", self.label);

        // Copy buffer to texture
        instance.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &buffer,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.size.0),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: self.size.0,
                height: self.size.1,
                depth_or_array_layers: 1,
            },
        );
    }
}
