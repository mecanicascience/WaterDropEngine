use bevy::{ecs::system::lifetimeless::SRes, prelude::*};
use wde_wgpu::instance::WRenderInstance;

use super::render_assets::{PrepareAssetError, RenderAsset};

/// The type of the material.
pub trait MaterialType: Send + Sync + Clone + TypePath + 'static {}


#[derive(Asset, TypePath, Clone)]
pub struct Material<M: MaterialType> {
    pub phantom: std::marker::PhantomData<M>,

    /// The label of the material.
    pub label: String,
}
impl<M: MaterialType> Default for Material<M> {
    fn default() -> Self {
        Material {
            phantom: std::marker::PhantomData,
            label: "material".to_string(),
        }
    }
}

pub struct GpuMaterial<M: MaterialType> {
    phantom: std::marker::PhantomData<M>,
}
impl<M: MaterialType> RenderAsset for GpuMaterial<M> {
    type SourceAsset = Material<M>;
    type Param = SRes<WRenderInstance<'static>>;

    fn prepare_asset(
            asset: Self::SourceAsset,
            _render_instance: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
        ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        debug!(asset.label, "Loading material on the GPU.");
        
        Ok(GpuMaterial {
            phantom: std::marker::PhantomData,
        })
    }
}
