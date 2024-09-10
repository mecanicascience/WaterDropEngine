mod mesh;
mod texture;
mod buffer;
mod shader;
mod material;
mod render_assets;

use bevy::prelude::*;

pub use mesh::*;
pub use texture::*;
pub use buffer::*;
pub use shader::*;
pub use material::*;
pub use render_assets::*;

use crate::core::RenderApp;

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
            .init_asset::<Shader>()
            .init_asset::<Buffer>();

        // Add resource loaders to transfer the assets to the GPU
        app
            .add_plugins(RenderAssetsPlugin::<GpuMesh>::default())
            .add_plugins(RenderAssetsPlugin::<GpuTexture>::default())
            .add_plugins(RenderAssetsPlugin::<GpuBuffer>::default());

        // Add cached resources
        app.get_sub_app_mut(RenderApp).unwrap()
            .init_resource::<MaterialsBuilderCache>();
    }
}
