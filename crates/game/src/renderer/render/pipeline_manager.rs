use std::collections::HashMap;

use bevy::{app::{App, Plugin}, asset::{AssetEvent, AssetId, Assets}, ecs::prelude::*, log::{error, warn}};
use wde_wgpu::{instance::WRenderInstance, render_pipeline::{ShaderStages, WRenderPipeline}};

use crate::{renderer::{extract_macros::ExtractWorld, Extract, Render, RenderSet}, scene::resources::Shader};

use super::RenderPipelineDescriptor;

/// The index of a cached pipeline.
pub type CachedPipelineIndex = usize;

/// The status of a cached pipeline.
pub enum CachedPipelineStatus<'a> {
    Loading,
    Ok(&'a WRenderPipeline),
    Error
}


pub struct PipelineManagerPlugin;
impl Plugin for PipelineManagerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PipelineManager>()
            .add_systems(Extract, extract_shaders)
            .add_systems(Render, load_pipelines.in_set(RenderSet::Prepare));
    }
}


#[derive(Resource, Default)]
pub struct PipelineManager {
    pub pipeline_iter: CachedPipelineIndex,

    pub processing_render_pipelines: HashMap<CachedPipelineIndex, RenderPipelineDescriptor>,
    pub loaded_render_pipelines: HashMap<CachedPipelineIndex, WRenderPipeline>,
    pub loaded_render_pipelines_desc: HashMap<CachedPipelineIndex, RenderPipelineDescriptor>,

    pub shader_cache: HashMap<AssetId<Shader>, Shader>,
    pub shader_to_pipelines: HashMap<AssetId<Shader>, Vec<CachedPipelineIndex>>,
}

impl PipelineManager {
    /// Push the creation of a render pipeline to the pipeline manager queue.
    /// 
    /// # Returns
    /// The index of the pipeline.
    pub fn create_render_pipeline(&mut self, descriptor: RenderPipelineDescriptor) -> CachedPipelineIndex {
        // Store the pipeline descriptor to the queued pipelines
        let id = self.pipeline_iter;
        self.processing_render_pipelines.insert(id, descriptor);
        self.pipeline_iter += 1;
        id
    }

    /// Get the status of a pipeline from its cached index.
    /// If the pipeline is loading, it will return `CachedPipelineStatus::Loading` with the pipeline being loaded.
    pub fn get_pipeline(&self, id: CachedPipelineIndex) -> CachedPipelineStatus {
        if self.processing_render_pipelines.contains_key(&id) {
            CachedPipelineStatus::Loading
        } else if let Some(pipeline) = self.loaded_render_pipelines.get(&id) {
            CachedPipelineStatus::Ok(pipeline)
        } else {
            error!("Pipeline with id {} not found", id);
            CachedPipelineStatus::Error
        }
    }
}

/// Extract the shaders from the asset server and store them in the pipeline manager.
fn extract_shaders(
    mut pipeline_manager: ResMut<PipelineManager>, shaders: ExtractWorld<Res<Assets<Shader>>>,
    mut shader_events: ExtractWorld<EventReader<AssetEvent<Shader>>>
) {
    let cache = &mut pipeline_manager.shader_cache;
    let mut updated_ids = Vec::new();
    for event in shader_events.read() {
        match event {
            AssetEvent::Added { id } => {
                if let Some(shader) = shaders.get(*id) {
                    cache.insert(*id, shader.clone());
                }
            }
            AssetEvent::Modified { id } => {
                if let Some(shader) = shaders.get(*id) {
                    cache.insert(*id, shader.clone());
                    updated_ids.push(*id);
                }
            }
            AssetEvent::Removed { id } => {
                cache.remove(id);
            }
            AssetEvent::Unused { .. } => {}
            AssetEvent::LoadedWithDependencies { .. } => {}
        }
    }

    // Recreate the shader to pipelines map
    for id in updated_ids {
        let p_ids = match pipeline_manager.shader_to_pipelines.get(&id) {
            Some(p_ids) => p_ids.clone(),
            None => continue
        };
        for p_id in p_ids.iter() {
            // Only update the pipeline if it is loaded
            if pipeline_manager.loaded_render_pipelines.contains_key(p_id) {
                let desc = pipeline_manager.loaded_render_pipelines_desc.remove(p_id).unwrap();
                pipeline_manager.processing_render_pipelines.insert(*p_id, desc.clone());
                pipeline_manager.loaded_render_pipelines.remove(p_id);
            }
        }
    }
}

/// Load the pipelines that are queued in the pipeline manager.
fn load_pipelines(
    mut pipeline_manager: ResMut<PipelineManager>,
    render_instance: Res<WRenderInstance<'static>>
) {
    let mut pipelines_loaded_indices: Vec<(usize, WRenderPipeline)> = Vec::new();
    let mut pipelines_loaded_desc: HashMap<CachedPipelineIndex, RenderPipelineDescriptor> = HashMap::new();
    let mut shaders_to_pipelines: HashMap<AssetId<Shader>, Vec<CachedPipelineIndex>> = pipeline_manager.shader_to_pipelines.clone();
    for (id, descriptor) in pipeline_manager.processing_render_pipelines.iter() {
        let mut can_load = true;

        // Check if vertex shader is loaded
        let vert_shader = match &descriptor.vert {
            Some(shader) => {
                match pipeline_manager.shader_cache.get(&shader.id()) {
                    Some(shader) => Some(shader),
                    None => {
                        // Shader is not loaded yet
                        can_load = false;
                        None
                    }
                }
            },
            None => None
        };

        // Check if fragment shader is loaded
        let frag_shader = match &descriptor.frag {
            Some(shader) => {
                match pipeline_manager.shader_cache.get(&shader.id()) {
                    Some(shader) => Some(shader),
                    None => {
                        // Shader is not loaded yet
                        can_load = false;
                        None
                    }
                }
            },
            None => None
        };

        // Skip if shaders are not loaded
        if !can_load {
            continue;
        }
        pipelines_loaded_desc.insert(*id, descriptor.clone());
        shaders_to_pipelines.entry(descriptor.vert.as_ref().unwrap().id()).or_default().push(*id);
        shaders_to_pipelines.entry(descriptor.frag.as_ref().unwrap().id()).or_default().push(*id);

        warn!("Loading pipeline: {}", descriptor.label);

        // Load the pipeline
        let mut pipeline = WRenderPipeline::new(descriptor.label);
        if let Some(vert_shader) = vert_shader {
            pipeline.set_shader(&vert_shader.content, ShaderStages::VERTEX);
        }
        if let Some(frag_shader) = frag_shader {
            pipeline.set_shader(&frag_shader.content, ShaderStages::FRAGMENT);
        }
        pipeline.set_topology(descriptor.topology);
        if descriptor.depth_stencil {
            pipeline.set_depth_stencil();
        }
        for push_constant in descriptor.push_constants.iter() {
            pipeline.add_push_constant(push_constant.stages, push_constant.offset, push_constant.size);
        }
        for _ in descriptor.bind_group_layouts.iter() {
            // pipeline.add_bind_group(*bind_group);
        }
        match pipeline.init(&render_instance.data.lock().unwrap()) {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to load pipeline: {:?}", e);
                continue;
            }
        }

        // Add the pipeline to the loaded pipelines
        pipelines_loaded_indices.push((*id, pipeline));
    }

    // Remove loaded pipelines and add them to the loaded pipelines
    while let Some((id, pipeline)) = pipelines_loaded_indices.pop() {
        pipeline_manager.processing_render_pipelines.remove(&id);
        pipeline_manager.loaded_render_pipelines.insert(id, pipeline);
        pipeline_manager.loaded_render_pipelines_desc.insert(id, pipelines_loaded_desc.remove(&id).unwrap());
    }

    // Update the shader to pipelines map
    pipeline_manager.shader_to_pipelines = shaders_to_pipelines;
}
