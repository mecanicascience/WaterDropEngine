use bevy::prelude::*;

use crate::core::{Render, RenderApp, RenderSet};

use super::{gizmo::GizmoRenderPass, pbr::{PbrGBufferRenderPass, PbrLightingRenderPass}, terrain::mc_render_core::MarchingCubesRenderPass};

pub struct RenderGraphPlugin;
impl Plugin for RenderGraphPlugin {
    fn build(&self, app: &mut App) {
        // Set the render graph
        app.get_sub_app_mut(RenderApp).unwrap()
            .add_systems(Render, (
                // PBR
                PbrGBufferRenderPass::render_g_buffer,
                PbrLightingRenderPass::render_lighting,

                // Terrain
                MarchingCubesRenderPass::render_terrain,

                // Gizmo
                GizmoRenderPass::render_gizmo
            ).chain().in_set(RenderSet::Render));
    }
}
