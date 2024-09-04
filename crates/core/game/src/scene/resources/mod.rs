mod mesh;
mod texture;

use bevy::prelude::*;

pub use mesh::*;
pub use texture::*;

use crate::renderer::render_assets::RenderAssetsPlugin;

pub struct SceneResourcesPlugin;
impl Plugin for SceneResourcesPlugin {
    fn build(&self, app: &mut App) {
        // Setup the assets
        app
            .init_asset_loader::<TextureLoader>()
            .init_asset::<Texture>()
            .init_asset_loader::<MeshLoader>()
            .init_asset::<Mesh>();
        // Add resource loaders
        app
            .add_plugins(RenderAssetsPlugin::<GpuMesh>::default())
            .add_plugins(RenderAssetsPlugin::<GpuTexture>::default());
    }
}
