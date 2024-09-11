use bevy::{ecs::system::lifetimeless::{SRes, SResMut}, prelude::*};
use wde_wgpu::render_pipeline::{WCompareFunction, WDepthStencilDescriptor};
use crate::{assets::{PrepareAssetError, RenderAsset}, pipelines::{CachedPipelineIndex, PipelineManager, RenderPipelineDescriptor}};

use super::PbrDeferredTexturesLayout;


#[derive(Default, Asset, Clone, TypePath)]
pub struct PbrLightingRenderPipeline;
pub struct GpuPbrLightingRenderPipeline {
    pub cached_pipeline_index: CachedPipelineIndex
}
impl RenderAsset for GpuPbrLightingRenderPipeline {
    type SourceAsset = PbrLightingRenderPipeline;
    type Param = (
        SRes<AssetServer>, SResMut<PipelineManager>, SRes<PbrDeferredTexturesLayout>
    );

    fn prepare_asset(
            asset: Self::SourceAsset,
            (
                assets_server, pipeline_manager,
                deferred_layout
            ): &mut bevy::ecs::system::SystemParamItem<Self::Param>
        ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        // Get the deferred layout
        let deferred_layout = match &deferred_layout.deferred_layout {
            Some(layout) => layout,
            None => return Err(PrepareAssetError::RetryNextUpdate(asset))
        };

        // Create the pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: "lighting-pbr",
            vert: Some(assets_server.load("pbr/lighting_vert.wgsl")),
            frag: Some(assets_server.load("pbr/lighting_frag.wgsl")),
            bind_group_layouts: vec![deferred_layout.clone()],
            depth: WDepthStencilDescriptor {
                enabled: true,
                write: false,
                compare: WCompareFunction::LessEqual
            },
            render_targets: None,
            ..Default::default()
        };
        let cached_index = pipeline_manager.create_render_pipeline(pipeline_desc);

        Ok(GpuPbrLightingRenderPipeline {
            cached_pipeline_index: cached_index
        })
    }
}
