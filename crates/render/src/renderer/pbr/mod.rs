use bevy::prelude::*;

mod pbr_material;
mod pbr_pipeline;
mod pbr_renderpass;
mod pbr_ssbo;

pub use pbr_material::*;
pub use pbr_pipeline::*;
pub use pbr_renderpass::*;
pub use pbr_ssbo::*;

use crate::{assets::{MaterialsPlugin, RenderAssetsPlugin}, core::{Extract, Render, RenderApp, RenderSet}};

use super::depth::DepthTexture;

pub(crate) struct PbrFeaturesPlugin;
impl Plugin for PbrFeaturesPlugin {
    fn build(&self, app: &mut App) {
        // Add the pbr material
        app
            .add_plugins(MaterialsPlugin::<PbrMaterial>::default());

        // Add the pbr ssbo
        app
            .add_plugins(PbrSsboPlugin);

        // Add the pbr pipeline
        app
            .init_asset::<PbrRenderPipeline>()
            .add_plugins(RenderAssetsPlugin::<GpuPbrRenderPipeline>::default());

        // Add the pbr render pass
        app.get_sub_app_mut(RenderApp).unwrap()
            .add_systems(Extract, (PbrRenderPass::create_batches, DepthTexture::extract_depth_texture))
            .add_systems(Render, PbrRenderPass::render.in_set(RenderSet::Render));
    }

    fn finish(&self, app: &mut App) {
        // Create the render pass
        app.get_sub_app_mut(RenderApp).unwrap()
            .insert_resource(PbrRenderPass {
                batches: Vec::new()
            });

        // Create the pipeline
        let pipeline: Handle<PbrRenderPipeline> = app.world_mut()
            .get_resource::<AssetServer>().unwrap().add(PbrRenderPipeline);
        app.get_sub_app_mut(RenderApp).unwrap().world_mut().spawn(pipeline);
    }
}

