use wde_resources::ResourceHandle;

/// Hold the terrain map.
#[derive(Debug, Clone)]
pub struct TerrainComponent {
    /// Heightmap of the terrain.
    pub heightmap: ResourceHandle
}
