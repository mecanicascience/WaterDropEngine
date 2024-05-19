use wde_resources::ResourceHandle;

/// Hold the terrain map.
#[derive(Debug, Clone)]
pub struct TerrainComponent {
    /// Heightmap of the terrain.
    pub heightmap: ResourceHandle,
    /// Number of chunks of the terrain.
    pub chunks: (u32, u32),
    /// Height of the terrain.
    pub height: f32,
}
