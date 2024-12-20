use std::collections::HashMap;

use bevy::{app::{App, Plugin}, asset::{AssetEvent, AssetId, Assets}, ecs::prelude::*, log::{debug, error}};
use wde_wgpu::{compute_pipeline::WComputePipeline, instance::WRenderInstance, render_pipeline::{WRenderPipeline, WShaderStages}};

use crate::{core::{extract_macros::ExtractWorld, Extract, Render, RenderSet}, assets::Shader};

use super::{RenderPipelineDescriptor, ComputePipelineDescriptor};

/// The index of a cached pipeline.
pub type CachedPipelineIndex = usize;

/// The status of a cached pipeline.
pub enum CachedPipelineStatus<'a> {
    Loading,
    OkRender(&'a WRenderPipeline),
    OkCompute(&'a WComputePipeline),
    Error
}


pub struct PipelineManagerPlugin;
impl Plugin for PipelineManagerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<PipelineManager>()
            .add_systems(Extract, extract_shaders)
            .add_systems(Render, (load_render_pipelines, load_compute_pipelines).in_set(RenderSet::Prepare));
    }
}


#[derive(Resource, Default)]
/// Stores the different pipelines in queue and loaded.
pub struct PipelineManager {
    pub pipeline_iter: CachedPipelineIndex,

    pub processing_render_pipelines: HashMap<CachedPipelineIndex, RenderPipelineDescriptor>,
    pub loaded_render_pipelines: HashMap<CachedPipelineIndex, WRenderPipeline>,
    pub loaded_render_pipelines_desc: HashMap<CachedPipelineIndex, RenderPipelineDescriptor>,

    pub processing_compute_pipelines: HashMap<CachedPipelineIndex, ComputePipelineDescriptor>,
    pub loaded_compute_pipelines: HashMap<CachedPipelineIndex, WComputePipeline>,
    pub loaded_compute_pipelines_desc: HashMap<CachedPipelineIndex, ComputePipelineDescriptor>,

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

    /// Push the creation of a compute pipeline to the pipeline manager queue.
    /// 
    /// # Returns
    /// The index of the pipeline.
    pub fn create_compute_pipeline(&mut self, descriptor: ComputePipelineDescriptor) -> CachedPipelineIndex {
        // Store the pipeline descriptor to the queued pipelines
        let id = self.pipeline_iter;
        self.processing_compute_pipelines.insert(id, descriptor);
        self.pipeline_iter += 1;
        id
    }

    /// Get the status of a pipeline from its cached index.
    /// If the pipeline is loading, it will return `CachedPipelineStatus::Loading` with the pipeline being loaded.
    pub fn get_pipeline(&self, id: CachedPipelineIndex) -> CachedPipelineStatus {
        if self.processing_render_pipelines.contains_key(&id) || self.processing_compute_pipelines.contains_key(&id) {
            CachedPipelineStatus::Loading
        } else if let Some(pipeline) = self.loaded_render_pipelines.get(&id) {
            CachedPipelineStatus::OkRender(pipeline)
        } else if let Some(pipeline) = self.loaded_compute_pipelines.get(&id) {
            CachedPipelineStatus::OkCompute(pipeline)
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
            if pipeline_manager.loaded_compute_pipelines.contains_key(p_id) {
                let desc = pipeline_manager.loaded_compute_pipelines_desc.remove(p_id).unwrap();
                pipeline_manager.processing_compute_pipelines.insert(*p_id, desc.clone());
                pipeline_manager.loaded_compute_pipelines.remove(p_id);
            }
        }
    }
}

/// Load the pipelines that are queued in the pipeline manager.
fn load_render_pipelines(
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

        debug!("Loading pipeline with id {}", id);

        // Build the layouts
        let mut bind_group_layouts = Vec::new();
        for layout in descriptor.bind_group_layouts.iter() {
            bind_group_layouts.push(layout.build(&render_instance.data.read().unwrap()));
        }

        // Load the pipeline
        let mut pipeline = WRenderPipeline::new(descriptor.label);
        if let Some(vert_shader) = vert_shader {
            pipeline.set_shader(&vert_shader.content, WShaderStages::VERTEX);
        }
        if let Some(frag_shader) = frag_shader {
            pipeline.set_shader(&frag_shader.content, WShaderStages::FRAGMENT);
        }
        pipeline.set_topology(descriptor.topology);
        pipeline.set_cull_mode(descriptor.cull_mode);
        pipeline.set_depth(descriptor.depth.clone());
        if let Some(ref render_targets) = descriptor.render_targets {
            pipeline.set_render_targets(render_targets.clone());
        }
        for push_constant in descriptor.push_constants.iter() {
            pipeline.add_push_constant(push_constant.stages, push_constant.offset, push_constant.size);
        }
        pipeline.set_bind_groups(bind_group_layouts);
        match pipeline.init(&render_instance.data.read().unwrap()) {
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

/// Load the pipelines that are queued in the pipeline manager.
fn load_compute_pipelines(
    mut pipeline_manager: ResMut<PipelineManager>,
    render_instance: Res<WRenderInstance<'static>>
) {
    let mut pipelines_loaded_indices: Vec<(usize, WComputePipeline)> = Vec::new();
    let mut pipelines_loaded_desc: HashMap<CachedPipelineIndex, ComputePipelineDescriptor> = HashMap::new();
    let mut shaders_to_pipelines: HashMap<AssetId<Shader>, Vec<CachedPipelineIndex>> = pipeline_manager.shader_to_pipelines.clone();
    for (id, descriptor) in pipeline_manager.processing_compute_pipelines.iter() {
        let mut can_load = true;

        // Check if compute shader is loaded
        let compute_shader = match &descriptor.comp {
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
        shaders_to_pipelines.entry(descriptor.comp.as_ref().unwrap().id()).or_default().push(*id);

        debug!("Loading pipeline with id {}", id);

        // Build the layouts
        let mut bind_group_layouts = Vec::new();
        for layout in descriptor.bind_group_layouts.iter() {
            bind_group_layouts.push(layout.build(&render_instance.data.read().unwrap()));
        }

        // Load the pipeline
        let mut pipeline = WComputePipeline::new(descriptor.label);
        if let Some(compute_shader) = compute_shader {
            pipeline.set_shader(&compute_shader.content);
        }
        pipeline.set_bind_groups(bind_group_layouts);
        match pipeline.init(&render_instance.data.read().unwrap()) {
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
        pipeline_manager.processing_compute_pipelines.remove(&id);
        pipeline_manager.loaded_compute_pipelines.insert(id, pipeline);
        pipeline_manager.loaded_compute_pipelines_desc.insert(id, pipelines_loaded_desc.remove(&id).unwrap());
    }

    // Update the shader to pipelines map
    pipeline_manager.shader_to_pipelines = shaders_to_pipelines;
}
