/// Describe the vertex structure of a mesh.
/// 
/// # Fields
/// 
/// * `position` - The position of the vertex (location 0).
/// * `tex_uv`   - The texture UV of the vertex (location 1).
/// * `normal`   - The normal of the vertex (location 2).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_uv: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    /// Describe the layout of the vertex.
    /// 
    /// # Returns
    /// 
    /// * `wgpu::VertexBufferLayout` - The layout of the vertex.
    pub fn describe<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
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
