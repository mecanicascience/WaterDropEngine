use bevy::prelude::*;

mod gizmo_pipeline;
mod gizmo_renderpass;
mod gizmo_ssbo;

pub use gizmo_pipeline::*;
pub use gizmo_renderpass::*;
pub use gizmo_ssbo::*;

use crate::{assets::RenderAssetsPlugin, core::{Extract, RenderApp}};

pub(crate) struct GizmoFeaturesPlugin;
impl Plugin for GizmoFeaturesPlugin {
    fn build(&self, app: &mut App) {
        // Add the gizmo ssbo
        app
            .add_plugins(GizmoSsboPlugin);

        // Add the pbr pipelines
        app
            .init_asset::<GizmoRenderPipeline>()
            .add_plugins(RenderAssetsPlugin::<GpuGizmoRenderPipeline>::default());

        // Add the pbr render passes
        app.get_sub_app_mut(RenderApp).unwrap()
            .add_systems(Extract,GizmoRenderPass::create_batches);
    }

    fn finish(&self, app: &mut App) {
        // Create the render pass
        app.get_sub_app_mut(RenderApp).unwrap()
            .insert_resource(GizmoRenderPass {
                batches_order: Default::default(),
                batches: Default::default()
            });

        // Create the gizmo pipeline
        let pipeline: Handle<GizmoRenderPipeline> = app.world_mut()
            .get_resource::<AssetServer>().unwrap().add(GizmoRenderPipeline);
        app.get_sub_app_mut(RenderApp).unwrap().world_mut().spawn(pipeline);
    }
}

