use wde_resources::ResourceHandle;

/// Store the rendering properties of a dynamic entity.
#[derive(Debug, Clone)]
pub struct RenderComponentDynamic {
    /// Unique identifier of the entity to render.
    pub id: u32,
    /// The model to use for rendering.
    pub model: ResourceHandle,
    /// The material to use for rendering.
    pub material: ResourceHandle
}

/// Store the rendering properties of a static entity.
#[derive(Debug, Clone)]
pub struct RenderComponentStatic {
    /// Unique identifier of the entity to render.
    pub id: u32,
    /// The model to use for rendering.
    pub model: ResourceHandle,
    /// The material to use for rendering.
    pub material: ResourceHandle
}
