use wde_logger::{debug, trace};
use wgpu::util::DeviceExt;

use crate::{RenderInstance, BindGroup, ShaderType, CommandBuffer};

/// Buffer usages.
pub enum BufferUsage {
    Vertex,
    Index,
    Uniform,
    Storage,
}

/// Buffer binding types.
pub enum BufferBindingType {
    Uniform,
    Storage { read_only: bool },
}

/// Create a buffer.
/// 
/// # Example
/// 
/// ```
/// let mut buffer = Buffer::new(&instance, "Buffer", 1024, BufferUsage::Vertex, None);
/// 
/// // Create a bind group for the buffer
/// let bind_group = buffer.create_bind_group(&instance, BufferBindingType::Uniform, wgpu::ShaderStages::VERTEX);
/// 
/// // Copy data from another buffer
/// buffer.copy_from_buffer(&instance, &[...]);
/// 
/// // Write data to the buffer
/// buffer.write(&instance, &[...], 0);
/// ```
pub struct Buffer {
    pub label: String,
    pub buffer: wgpu::Buffer,
}

impl Buffer {
    /// Create a new buffer.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `label` - The label of the buffer.
    /// * `size` - The size of the buffer.
    /// * `usage` - The usage of the buffer (vertex, index, uniform, storage).
    /// * `content` - The content of the buffer.
    pub fn new(instance: &RenderInstance, label: &str, size: usize, usage: BufferUsage, content: Option<&[u8]>) -> Self {
        debug!("Creating '{}' Buffer.", label);

        // In case the content is not provided, create an empty buffer.
        match content {
            Some(content) => {
                // Create buffer
                let buffer = instance.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some(format!("'{}' Buffer", label).as_str()),
                        contents: content,
                        usage: match usage {
                            BufferUsage::Vertex => wgpu::BufferUsages::VERTEX,
                            BufferUsage::Index => wgpu::BufferUsages::INDEX,
                            BufferUsage::Uniform => wgpu::BufferUsages::UNIFORM,
                            BufferUsage::Storage => wgpu::BufferUsages::STORAGE,
                        },
                    }
                );
                
                Buffer {
                    label: label.to_string(),
                    buffer,
                }
            },
            None => {
                // Create empty buffer of the given size
                let buffer = instance.device.create_buffer(
                    &wgpu::BufferDescriptor {
                        label: Some(format!("'{}' Buffer", label).as_str()),
                        size: size as u64,
                        usage: match usage {
                            BufferUsage::Vertex => wgpu::BufferUsages::VERTEX,
                            BufferUsage::Index => wgpu::BufferUsages::INDEX,
                            BufferUsage::Uniform => wgpu::BufferUsages::UNIFORM,
                            BufferUsage::Storage => wgpu::BufferUsages::STORAGE,
                        },
                        mapped_at_creation: false,
                    }
                );

                Buffer {
                    label: label.to_string(),
                    buffer,
                }
            },
        }
    }

    /// Create a bind group for the buffer.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `binding_type` - The type of the buffer.
    /// * `visibility` - The list of shader stages that can access the buffer.
    /// 
    /// # Returns
    /// 
    /// * `BindGroup` - The bind group of the buffer.
    pub async fn create_bind_group(&mut self, instance: &RenderInstance, binding_type: BufferBindingType, visibility: ShaderType) -> BindGroup {
        trace!("Creating bind group for '{}' Buffer.", self.label);
        
        // Create bind group layout
        let layout_entries = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: match visibility {
                    ShaderType::Vertex => wgpu::ShaderStages::VERTEX,
                    ShaderType::Fragment => wgpu::ShaderStages::FRAGMENT
                },
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: match binding_type {
                        BufferBindingType::Uniform => wgpu::BufferBindingType::Uniform,
                        BufferBindingType::Storage { read_only } => wgpu::BufferBindingType::Storage { read_only },
                    },
                },
                count: None,
            }
        ];

        let bind_group = tokio::task::block_in_place(|| {
            let layout = instance.device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some(format!("'{}' Buffer Bind Group Layout", self.label).as_str()),
                    entries: &layout_entries,
                }
            );

            // Create bind group
            instance.device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some(format!("'{}' Buffer Bind Group", self.label).as_str()),
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.buffer.as_entire_binding(),
                        }
                    ],
                }
            )
        });
        

        // Return bind group
        BindGroup::new(
            self.label.clone(),
            bind_group
        )
    }


    /// Copy data to the buffer from another buffer.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `buffer` - The buffer to copy from.
    pub async fn copy_from_buffer(&mut self, instance: &RenderInstance, buffer: &Buffer) {
        trace!("Copying from '{}' to '{}' Buffer.", buffer.label, self.label);
        
        // Create command encoder
        let mut command_buffer = CommandBuffer::new(
            instance,
            &format!("Copy from '{}' to '{}' Buffer", buffer.label, self.label)).await;

        // Copy buffer
        command_buffer.copy_buffer_to_buffer(&buffer, &self);

        // Submit commands
        command_buffer.submit(&instance);
    }

    /// Write data to the buffer.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `content` - The content to write to the buffer.
    /// * `offset` - The offset to write the content to.
    pub fn write(&mut self, instance: &RenderInstance, content: &[u8], offset: usize) {
        trace!("Writing to '{}' Buffer.", self.label);

        instance.queue.write_buffer(
            &self.buffer,
            offset as u64,
            content);
    }
}
