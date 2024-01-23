use wde_resources::ResourceHandle;

/// Store the rendering properties of a dynamic entity.
#[derive(Debug, Clone)]
pub struct RenderComponentDynamic {
    /// The model to use for rendering.
    pub model: ResourceHandle
}

/// Store the rendering properties of a static entity.
#[derive(Debug, Clone)]
pub struct RenderComponentStatic {
    /// The model to use for rendering.
    pub model: ResourceHandle
}
