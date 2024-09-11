use bevy::prelude::*;
use depth::DepthTexture;
use pbr::PbrFeaturesPlugin;

use crate::core::{Extract, RenderApp};

pub mod pbr;
pub mod depth;

pub(crate) struct RendererPlugin;
impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PbrFeaturesPlugin)
            .add_systems(Startup, DepthTexture::init_depth)
            .add_systems(Update, DepthTexture::resize_depth);
        app.get_sub_app_mut(RenderApp).unwrap()
            .add_systems(Extract, DepthTexture::extract_depth_texture);
    }
}
