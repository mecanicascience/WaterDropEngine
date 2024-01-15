use wde_resources::ResourceHandle;

/// Store the rendering properties of a dynamic entity.
pub struct RenderComponentDynamic {
    /// The model to use for rendering.
    pub model: ResourceHandle
}

/// Store the rendering properties of a static entity.
pub struct RenderComponentStatic {
    /// The model to use for rendering.
    pub model: ResourceHandle
}
