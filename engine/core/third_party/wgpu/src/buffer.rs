use std::fmt::Formatter;

use tracing::warn;
use wde_logger::{trace, info};
use wgpu::{util::DeviceExt, BindGroupLayout};

use crate::{RenderInstance, BindGroup, CommandBuffer};

/// Buffer usages.
pub type BufferUsage = wgpu::BufferUsages;

/// Shader stages.
pub type ShaderStages = wgpu::ShaderStages;

/// Buffer binding types.
pub type BufferBindingType = wgpu::BufferBindingType;

/// Map the buffer.
pub type BufferViewMut<'a> = wgpu::BufferViewMut<'a>;

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
/// 
/// // Map the buffer
/// buffer.map_mut(|range| {
///    [...]
/// });
/// ```
pub struct Buffer {
    pub label: String,
    pub buffer: wgpu::Buffer,
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("label", &self.label)
            .field("buffer_size", &self.buffer.size())
            .finish()
    }
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
        info!(label, "Creating new buffer.");

        // In case the content is not provided, create an empty buffer.
        match content {
            Some(content) => {
                // Create buffer
                let buffer = instance.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some(format!("'{}' Buffer", label).as_str()),
                        contents: content,
                        usage 
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
                        usage,
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

    /// Create a bind group layout for the buffer.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `binding_type` - The type of the buffer.
    /// * `visibility` - The list of shader stages that can access the buffer.
    /// 
    /// # Returns
    /// 
    /// * `BindGroupLayout` - The bind group layout of the buffer.
    pub async fn create_bind_group_layout(&mut self, instance: &RenderInstance, binding_type: BufferBindingType, visibility: ShaderStages) -> BindGroupLayout {
        // Create bind group layout
        let layout_entries = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: binding_type,
                },
                count: None,
            }
        ];

        instance.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some(format!("'{}' Buffer Bind Group Layout", self.label).as_str()),
                entries: &layout_entries,
            }
        )
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
    pub async fn create_bind_group(&mut self, instance: &RenderInstance, binding_type: BufferBindingType, visibility: ShaderStages) -> BindGroup {
        // Create bind group layout
        let layout = self.create_bind_group_layout(instance, binding_type, visibility).await;

        // Create bind group
        let bind_group = instance.device.create_bind_group(
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
        );

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
        trace!(src=buffer.label, dest=self.label, "Copying data from buffer to buffer.");
        
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
        trace!(self.label, "Writing to buffer.");

        instance.queue.write_buffer(
            &self.buffer,
            offset as u64,
            content);
    }

    /// Map the buffer as mutable.
    /// This will wait for the buffer to be mapped.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `callback` - A closure that takes a mutable reference to the buffer.
    /// The callback takes a mutable reference to the buffer.
    pub fn map_mut(&self, instance: &RenderInstance, callback: impl FnOnce(BufferViewMut)) {
        trace!(self.label, "Mapping buffer.");

        // Map buffer
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = self.buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Write,
            move |r| sender.send(r).unwrap());

        // Wait for the buffer to be mapped
        instance.device.poll(wgpu::Maintain::Wait);
        receiver.recv().unwrap().unwrap();
        trace!(self.label, "Buffer mapped.");

        // Call callback
        callback(buffer_slice.get_mapped_range_mut());

        // Unmap buffer
        self.buffer.unmap();
        trace!(self.label, "Buffer unmapped.");
    }
}

impl Drop for Buffer {
    #[tracing::instrument]
    fn drop(&mut self) {
        info!(self.label, "Dropping buffer.");
    }
}
