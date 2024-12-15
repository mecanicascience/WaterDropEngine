use bevy::{ecs::system::lifetimeless::{SRes, SResMut}, prelude::*};
use wde_wgpu::render_pipeline::WDepthStencilDescriptor;
use crate::{assets::{materials::GizmoMaterial, GpuMaterial, PrepareAssetError, RenderAsset, RenderAssets}, features::CameraFeatureRender, pipelines::{CachedPipelineIndex, PipelineManager, RenderPipelineDescriptor}};

use super::GizmoSsbo;


#[derive(Default, Asset, Clone, TypePath)]
pub struct GizmoRenderPipeline;
pub struct GpuGizmoRenderPipeline {
    pub cached_pipeline_index: CachedPipelineIndex
}
impl RenderAsset for GpuGizmoRenderPipeline {
    type SourceAsset = GizmoRenderPipeline;
    type Param = (
        SRes<AssetServer>, SResMut<PipelineManager>, SRes<CameraFeatureRender>,
        SRes<RenderAssets<GpuMaterial<GizmoMaterial>>>, SRes<GizmoSsbo>
    );

    fn prepare_asset(
            asset: Self::SourceAsset,
            (
                assets_server, pipeline_manager,
                camera_feature, materials, ssbo
            ): &mut bevy::ecs::system::SystemParamItem<Self::Param>
        ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {

        // Get the ssbo layout
        let ssbo_layout = match &ssbo.bind_group_layout {
            Some(layout) => layout,
            None => return Err(PrepareAssetError::RetryNextUpdate(asset))
        };

        // Get the material layout
        let material = match materials.iter().next() {
            Some((_, material)) => material,
            None => return Err(PrepareAssetError::RetryNextUpdate(asset))
        };

        // Create the pipeline
        let pipeline_desc = RenderPipelineDescriptor {
            label: "gizmo",
            vert: Some(assets_server.load("gizmo/vert.wgsl")),
            frag: Some(assets_server.load("gizmo/frag.wgsl")),
            bind_group_layouts: vec![camera_feature.layout.clone(), ssbo_layout.clone(), material.bind_group_layout.clone()],
            depth: WDepthStencilDescriptor {
                enabled: true,
                ..Default::default()
            },
            topology: wde_wgpu::render_pipeline::WTopology::LineList,
            cull_mode: None,
            ..Default::default()
        };
        let cached_index = pipeline_manager.create_render_pipeline(pipeline_desc);

        Ok(GpuGizmoRenderPipeline {
            cached_pipeline_index: cached_index
        })
    }

    fn label(&self) -> &str {
        "gizmo"
    }
}
