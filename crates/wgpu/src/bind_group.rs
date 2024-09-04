//! Bind groups are used to bind resources to shaders.

use bevy::log::debug;

use crate::{buffer::WBuffer, instance::WRenderInstanceData, texture::WTexture};

/// Builder for a bind group.
/// Use the `new()` function to create a new bind group builder.
pub struct WBindGroupBuilder<'a> {
    pub label: String,
    layout_entries: Vec<wgpu::BindGroupLayoutEntry>,
    group_entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl std::fmt::Debug for WBindGroupBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BindGroupBuilder")
            .field("label", &self.label)
            .field("layout_entries", &self.layout_entries)
            .field("group_entries", &self.group_entries)
            .finish()
    }
}

impl<'a> WBindGroupBuilder<'a> {
    /// Create a new bind group builder.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the bind group.
    pub fn new(label: &str) -> Self {
        WBindGroupBuilder {
            label: label.to_string(),
            layout_entries: Vec::new(),
            group_entries: Vec::new(),
        }
    }

    /// Add a buffer to the bind group.
    /// 
    /// # Arguments
    /// 
    /// * `binding` - The binding index of the buffer.
    /// * `buffer` - The buffer to add to the bind group.
    /// * `visibility` - The shader stages that can access the buffer.
    /// * `binding_type` - The type of the buffer binding.
    pub fn add_buffer<'b>(&mut self, binding: u32, buffer: &'b WBuffer, visibility: wgpu::ShaderStages, binding_type: wgpu::BufferBindingType) -> &mut Self where 'b: 'a {
        // Create bind group layout
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                has_dynamic_offset: false,
                min_binding_size: None,
                ty: binding_type,
            },
            count: None,
        });

        // Create bind group
        self.group_entries.push(wgpu::BindGroupEntry {
            binding,
            resource: buffer.buffer.as_entire_binding(),
        });

        self
    }

    /// Add a texture to the bind group.
    /// 
    /// # Arguments
    /// 
    /// * `binding` - The binding index of the texture. Note that the binding index of the sampler is incremented by 1.
    /// * `texture` - The texture to add to the bind group.
    /// * `visibility` - The shader stages that can access the texture.
    pub fn add_texture<'b>(&mut self, binding: u32, texture: &'b WTexture, visibility: wgpu::ShaderStages) -> &mut Self where 'b: 'a {
        // Create bind group layout
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None
        });
        self.layout_entries.push(wgpu::BindGroupLayoutEntry {
            binding: binding + 1,
            visibility,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        });

        // Create bind group
        self.group_entries.push(wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(&texture.view),
        });
        self.group_entries.push(wgpu::BindGroupEntry {
            binding: binding + 1,
            resource: wgpu::BindingResource::Sampler(&texture.sampler),
        });

        self
    }
}

impl Clone for WBindGroupBuilder<'_> {
    fn clone(&self) -> Self {
        WBindGroupBuilder {
            label: self.label.clone(),
            layout_entries: self.layout_entries.clone(),
            group_entries: self.group_entries.clone(),
        }
    }
}



/// Structure for a bind group.
/// 
/// # Example
/// 
/// ```
/// // Create a new bind group builder
/// let mut bind_group_builder = WBindGroupBuilder::new("Bind Group");
/// 
/// // Add a buffer to the bind group
/// bind_group_builder.add_buffer(0, &buffer, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Uniform);
/// 
/// // Add a texture to the bind group. Note that the binding is incremented by 1.
/// bind_group_builder.add_texture(1, &texture, wgpu::ShaderStages::FRAGMENT);
/// 
/// // Build the bind group
/// let bind_group = WBindGroup::new(&instance, bind_group_builder.clone());
/// ```
#[derive(Debug)]
pub struct WBindGroup {
    // Bind group description
    pub label: String,
    
    // Access to the data layout and group
    pub layout: wgpu::BindGroupLayout,
    pub group: wgpu::BindGroup,
}

impl WBindGroup {
    /// Creates a new bind group.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `builder` - The bind group builder.
    pub fn new(instance: &WRenderInstanceData, builder: WBindGroupBuilder) -> Self {
        debug!(builder.label, "Creating bind group.");

        // Create bind group layout
        let layout = instance.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some(format!("{}-buffer-bg-layout", builder.label).as_str()),
                entries: &builder.layout_entries,
            }
        );

        // Create bind group
        let group = instance.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some(format!("{}-buffer-bg", builder.label).as_str()),
                layout: &layout,
                entries: &builder.group_entries,
            }
        );

        // Create bind group
        WBindGroup {
            label: builder.label.clone(),
            layout,
            group,
        }
    }
}
