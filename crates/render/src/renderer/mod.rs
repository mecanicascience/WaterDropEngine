use bevy::prelude::*;
use depth::{DepthTexture, DepthTextureLayout};
use gizmo::{GizmoFeaturesPlugin, GizmoRenderPass};
use pbr::{PbrFeaturesPlugin, PbrGBufferRenderPass, PbrLightingRenderPass};

use crate::core::{Extract, Render, RenderApp, RenderSet};

pub mod pbr;
pub mod depth;
pub mod gizmo;

pub(crate) struct RendererPlugin;
impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        // Add the depth texture to the app
        app
            .add_systems(Startup, DepthTexture::create_texture)
            .add_systems(Update, DepthTexture::resize_texture);
        app.get_sub_app_mut(RenderApp).unwrap()
            .init_resource::<DepthTextureLayout>()
            .add_systems(Extract, DepthTexture::extract_texture)
            .add_systems(Render, DepthTextureLayout::build_bind_group.in_set(RenderSet::BindGroups));

        // Set the render graph
        app.get_sub_app_mut(RenderApp).unwrap()
            .add_systems(Render, (
                PbrGBufferRenderPass::render_g_buffer,
                PbrLightingRenderPass::render_lighting,
                GizmoRenderPass::render_gizmo
            ).chain().in_set(RenderSet::Render));

        // Add the materials
        app
            .add_plugins(PbrFeaturesPlugin)
            .add_plugins(GizmoFeaturesPlugin);
    }
}
