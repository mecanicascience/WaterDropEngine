use bevy::{ecs::system::lifetimeless::{SRes, SResMut}, prelude::*};
use crate::{assets::{PrepareAssetError, RenderAsset}, features::CameraFeatureRender, pipelines::{CachedPipelineIndex, PipelineManager, RenderPipelineDescriptor}};
use wde_wgpu::render_pipeline::WDepthStencilDescriptor;


#[derive(Default, Asset, Clone, TypePath)]
pub struct MarchingCubesRenderPipeline;
pub struct GpuMarchingCubesRenderPipeline {
    pub cached_pipeline_index: CachedPipelineIndex
}
impl RenderAsset for GpuMarchingCubesRenderPipeline {
    type SourceAsset = MarchingCubesRenderPipeline;
    type Param = (SRes<AssetServer>, SResMut<PipelineManager>, SRes<CameraFeatureRender>);

    fn prepare_asset(
            _asset: Self::SourceAsset,
            (
                assets_server, pipeline_manager,
                camera_feature
            ): &mut bevy::ecs::system::SystemParamItem<Self::Param>
        ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        // Create the pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: "marching-cubes",
            vert: Some(assets_server.load("marching-cubes/render.vert.wgsl")),
            frag: Some(assets_server.load("marching-cubes/render.frag.wgsl")),
            bind_group_layouts: vec![camera_feature.layout.clone()],
            depth: WDepthStencilDescriptor {
                enabled: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let cached_index = pipeline_manager.create_render_pipeline(pipeline_desc);

        Ok(GpuMarchingCubesRenderPipeline {
            cached_pipeline_index: cached_index
        })
    }

    fn label(&self) -> &str {
        "marching-cubes"
    }
}
