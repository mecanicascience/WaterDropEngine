mod mesh;
mod texture;
mod shader;

pub mod render_assets;

use bevy::prelude::*;
use render_assets::RenderAssetsPlugin;

pub use mesh::*;
pub use texture::*;
pub use shader::*;

pub struct SceneResourcesPlugin;
impl Plugin for SceneResourcesPlugin {
    fn build(&self, app: &mut App) {
        // Setup the assets
        app
            .init_asset_loader::<TextureLoader>()
            .init_asset::<Texture>()
            .init_asset_loader::<MeshLoader>()
            .init_asset::<Mesh>()
            .init_asset_loader::<ShaderLoader>()
            .init_asset::<Shader>();

        // Add resource loaders to transfer the assets to the GPU
        app
            .add_plugins(RenderAssetsPlugin::<GpuMesh>::default())
            .add_plugins(RenderAssetsPlugin::<GpuTexture>::default());
    }
}
