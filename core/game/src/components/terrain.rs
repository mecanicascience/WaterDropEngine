use wde_resources::ResourceHandle;

/// Hold the terrain map.
#[derive(Debug, Clone)]
pub struct TerrainComponent {
    /// Heightmap of the terrain.
    pub heightmap: ResourceHandle,
    /// World scale of the terrain.
    pub scale: (f32, f32),
    /// Subdivision count of the terrain.
    pub subdivision: u32,
}
