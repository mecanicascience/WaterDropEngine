//! Vertex structure and layout for a mesh.

/// Describe the vertex structure of a mesh.
/// 
/// # Fields
/// 
/// * `position` - The position of the vertex (location 0).
/// * `uv`       - The texture UV of the vertex (location 1).
/// * `normal`   - The normal of the vertex (location 2).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct WVertex {
    /// The position of the vertex.
    pub position: [f32; 3],
    /// The texture UV of the vertex (must be between 0.0 and 1.0).
    pub uv: [f32; 2],
    /// The normal of the vertex (must be normalized).
    pub normal: [f32; 3],
}

impl WVertex {
    /// Describe the layout of the vertex.
    /// 
    /// # Returns
    /// 
    /// * `wgpu::VertexBufferLayout` - The layout of the vertex.
    pub fn describe<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<WVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute { // Position
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute { // UV
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute { // Normal
                    offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
