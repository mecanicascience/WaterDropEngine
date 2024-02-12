use std::fmt::Formatter;

use tracing::warn;
use wde_logger::info;
use wgpu::{util::DeviceExt, BufferView};

use crate::{RenderInstance, CommandBuffer};

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


    /// Copy data to the buffer from another buffer.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `buffer` - The buffer to copy from.
    pub async fn copy_from_buffer(&mut self, instance: &RenderInstance<'_>, buffer: &Buffer) {
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
        instance.queue.write_buffer(
            &self.buffer,
            offset as u64,
            content);
    }

    /// Map the buffer.
    /// This will wait for the buffer to be mapped.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `callback` - A closure that takes a reference to the buffer.
    /// The callback takes reference to the buffer.
    pub fn map(&self, instance: &RenderInstance, callback: impl FnOnce(BufferView)) {
        // Map buffer
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = self.buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read,
            move |r| sender.send(r).unwrap());

        // Wait for the buffer to be mapped
        instance.device.poll(wgpu::Maintain::Wait);
        receiver.recv().unwrap().unwrap();

        // Call callback
        callback(buffer_slice.get_mapped_range());

        // Unmap buffer
        self.buffer.unmap();
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
        // Map buffer
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = self.buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Write,
            move |r| sender.send(r).unwrap());

        // Wait for the buffer to be mapped
        instance.device.poll(wgpu::Maintain::Wait);
        receiver.recv().unwrap().unwrap();

        // Call callback
        callback(buffer_slice.get_mapped_range_mut());

        // Unmap buffer
        self.buffer.unmap();
    }
}

impl Drop for Buffer {
    #[tracing::instrument]
    fn drop(&mut self) {
        info!(self.label, "Dropping buffer.");
    }
}
