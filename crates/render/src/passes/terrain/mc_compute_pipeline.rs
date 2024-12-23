use bevy::{ecs::system::lifetimeless::{SRes, SResMut}, prelude::*};
use crate::{assets::{PrepareAssetError, RenderAsset}, pipelines::{CachedPipelineIndex, ComputePipelineDescriptor, PipelineManager}};
use wde_wgpu::{bind_group::BindGroupLayout, buffer::BufferBindingType, render_pipeline::WShaderStages};

#[derive(Default, Asset, Clone, TypePath)]
pub struct MarchingCubesComputePipelineAsset;
#[derive(Component)]
pub struct MarchingCubesComputePipeline(pub Handle<MarchingCubesComputePipelineAsset>);
pub struct GpuMarchingCubesComputePipeline {
    pub cached_pipeline_index: CachedPipelineIndex,
    pub desc_gpu_layout: Option<BindGroupLayout>,
    pub points_layout: Option<BindGroupLayout>,
    pub vertices_layout: Option<BindGroupLayout>
}
impl RenderAsset for GpuMarchingCubesComputePipeline {
    type SourceAsset = MarchingCubesComputePipelineAsset;
    type Param = (
        SRes<AssetServer>, SResMut<PipelineManager>
    );

    fn prepare_asset(
            _asset: Self::SourceAsset,
            (
                assets_server, pipeline_manager
            ): &mut bevy::ecs::system::SystemParamItem<Self::Param>
        ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        // Create the layouts
        let desc_gpu_layout = BindGroupLayout::new("marching-cubes-desc", |builder| {
            builder.add_buffer(0,
                WShaderStages::COMPUTE,
                BufferBindingType::Storage { read_only: false });
        });
        let points_layout = BindGroupLayout::new("points-marching-cubes", |builder| {
            builder.add_buffer(0,
                WShaderStages::COMPUTE,
                BufferBindingType::Storage { read_only: true });
        });
        let vertices_layout = BindGroupLayout::new("vertices-marching-cubes", |builder| {
            builder.add_buffer(0,
                WShaderStages::COMPUTE,
                BufferBindingType::Storage { read_only: false });
            builder.add_buffer(1,
                WShaderStages::COMPUTE,
                BufferBindingType::Storage { read_only: false });
        });

        // Create the pipeline
        let pipeline_desc = ComputePipelineDescriptor {
            label: "marching-cubes",
            comp: Some(assets_server.load("marching-cubes/marching_cube.comp.wgsl")),
            bind_group_layouts: vec![desc_gpu_layout.clone(), points_layout.clone(), vertices_layout.clone()],
            ..Default::default()
        };
        let cached_index = pipeline_manager.create_compute_pipeline(pipeline_desc);

        Ok(GpuMarchingCubesComputePipeline {
            cached_pipeline_index: cached_index,
            desc_gpu_layout: Some(desc_gpu_layout),
            points_layout: Some(points_layout),
            vertices_layout: Some(vertices_layout)
        })
    }

    fn label(&self) -> &str {
        "marching-cubes"
    }
}
