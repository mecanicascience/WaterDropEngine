use bevy::prelude::*;

mod pbr_material;
mod pbr_pipeline_gbuffer;
mod pbr_renderpass_gbuffer;
mod pbr_pipeline_lighting;
mod pbr_renderpass_lighting;
mod pbr_ssbo;
mod pbr_textures;

pub use pbr_material::*;
pub use pbr_pipeline_gbuffer::*;
pub use pbr_renderpass_gbuffer::*;
pub use pbr_pipeline_lighting::*;
pub use pbr_renderpass_lighting::*;
pub use pbr_ssbo::*;
pub use pbr_textures::*;

use crate::{assets::{MaterialsPlugin, RenderAssetsPlugin}, core::{Extract, Render, RenderApp, RenderSet}};

pub(crate) struct PbrFeaturesPlugin;
impl Plugin for PbrFeaturesPlugin {
    fn build(&self, app: &mut App) {
        // Add the pbr material
        app
            .add_plugins(MaterialsPlugin::<PbrMaterial>::default());

        // Add the pbr ssbo
        app
            .add_plugins(PbrSsboPlugin);

        // Add the pbr defered textures
        app
            .add_systems(Startup, PbrDeferredTextures::create_textures)
            .add_systems(Update, PbrDeferredTextures::resize_textures);
        app.get_sub_app_mut(RenderApp).unwrap()
            .init_resource::<PbrDeferredTexturesLayout>()
            .add_systems(Extract, PbrDeferredTextures::extract_textures)
            .add_systems(Render, PbrDeferredTexturesLayout::build_bind_group.in_set(RenderSet::BindGroups));

        // Add the pbr pipelines
        app
            .init_asset::<PbrGBufferRenderPipeline>()
            .add_plugins(RenderAssetsPlugin::<GpuPbrGBufferRenderPipeline>::default())
            .init_asset::<PbrLightingRenderPipeline>()
            .add_plugins(RenderAssetsPlugin::<GpuPbrLightingRenderPipeline>::default());

        // Add the pbr render passes
        app
            .init_resource::<PbrLightingRenderPassMesh>()
            .add_systems(Startup, PbrLightingRenderPassMesh::init);
        app.get_sub_app_mut(RenderApp).unwrap()
            .init_resource::<PbrLightingRenderPassMesh>()
            .add_systems(Extract,
                (PbrLightingRenderPassMesh::extract_mesh, PbrGBufferRenderPass::create_batches)
            .chain())
            .add_systems(Render, (
                PbrGBufferRenderPass::render_g_buffer, PbrLightingRenderPass::render_lighting
            ).chain().in_set(RenderSet::Render));
    }

    fn finish(&self, app: &mut App) {
        // Create the render pass
        app.get_sub_app_mut(RenderApp).unwrap()
            .insert_resource(PbrGBufferRenderPass {
                batches: Vec::new()
            });

        // Create the gbuffer pipeline
        let pipeline: Handle<PbrGBufferRenderPipeline> = app.world_mut()
            .get_resource::<AssetServer>().unwrap().add(PbrGBufferRenderPipeline);
        app.get_sub_app_mut(RenderApp).unwrap().world_mut().spawn(pipeline);

        // Create the lighting pipeline
        let pipeline: Handle<PbrLightingRenderPipeline> = app.world_mut()
            .get_resource::<AssetServer>().unwrap().add(PbrLightingRenderPipeline);
        app.get_sub_app_mut(RenderApp).unwrap().world_mut().spawn(pipeline);
    }
}

