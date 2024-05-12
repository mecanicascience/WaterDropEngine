use std::ops::Range;

use wde_resources::ResourceHandle;

/// Store the rendering properties of an entity.
#[derive(Debug, Clone)]
pub struct RenderComponent {
    /// Unique identifier of the entity to render in the SSBO.
    pub id: u32,
    /// The model to use for rendering.
    pub model: Option<ResourceHandle>,
    /// The material to use for rendering.
    pub material: Option<ResourceHandle>
}

/// Store the rendering properties of a list of entities for instanced rendering.
/// Note that this parent entity will not be rendered.
#[derive(Debug, Clone)]
pub struct RenderComponentInstanced {
    /// List of unique identifiers of the entities to render in the SSBO.
    pub ids: Range<u32>,
    /// The model to use for rendering.
    pub model: Option<ResourceHandle>,
    /// The material to use for rendering.
    pub material: Option<ResourceHandle>
}

/// Store the rendering properties of a child entity.
#[derive(Debug, Clone)]
pub struct RenderComponentChild {}



/// This will update the SSBO with the new data for the entity every frame.
#[derive(Debug, Clone)]
pub struct RenderComponentSSBODynamic {
    /// Unique identifier of the entity in the SSBO.
    pub id: u32
}

/// This will update the SSBO with the new data for the entity once.
#[derive(Debug, Clone)]
pub struct RenderComponentSSBOStatic {
    /// Unique identifier of the entity in the SSBO.
    pub id: u32
}
