use bevy::prelude::*;
use depth::DepthTexture;
use pbr::PbrFeaturesPlugin;

pub mod pbr;
pub mod depth;

pub(crate) struct RendererPlugin;
impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PbrFeaturesPlugin)
            .add_systems(Startup, DepthTexture::init_depth)
            .add_systems(Update, DepthTexture::resize_depth);
    }
}
