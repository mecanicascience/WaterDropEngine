use std::{any::Any, sync::{Arc, Mutex}};

use tracing::{error, warn};
use wde_logger::info;
use wde_wgpu::{RenderInstance, RenderPipeline, ShaderType};

use crate::{LoadedFlag, Resource, ResourceDescription, ResourceHandle, ResourceType, ResourcesManager, ShaderResource};

/// Temporary data to be transferred.
#[derive(Clone, Debug)]
struct TempMaterialData {
    vert_shader: ResourceHandle,
    frag_shader: ResourceHandle,
}

/// Resource data.
#[derive(Debug)]
pub struct MaterialData {
    pub pipeline: RenderPipeline,
}

/// Store a material resource loaded.
#[derive(Debug)]
pub struct MaterialResource {
    /// Label of the material.
    pub label: String,
    /// Data of the material.
    pub data: Option<MaterialData>,
    /// Loaded flag
    loaded: bool,
    /// Temporary data to be transferred.
    temp_data: Option<TempMaterialData>,

    // Async loading
    async_loaded: LoadedFlag,
}

impl Resource for MaterialResource {
    #[tracing::instrument]
    fn new(desc: ResourceDescription) -> Self where Self: Sized {
        info!(desc.label, "Creating material resource.");
        
        // Check if resource type is correct
        if desc.resource_type != Self::resource_type() {
            error!(desc.label, "Trying to create a material resource with a non material resource description.");
            return Self {
                label: desc.label.to_string(),
                data: None,
                loaded: false,
                async_loaded: LoadedFlag { flag: Arc::new(Mutex::new(false)), },
                temp_data: None
            };
        }

        // Create async loaded flag
        let async_loaded = LoadedFlag { flag: Arc::new(Mutex::new(true)), };

        // Identify shaders
        let mut vert_shader = None;
        let mut frag_shader = None;
        for dep in desc.dependencies.iter() {
            match dep.resource_type {
                ResourceType::Shader => {
                    if dep.label.contains("vert") {
                        vert_shader = Some(dep.clone());
                    } else if dep.label.contains("frag") {
                        frag_shader = Some(dep.clone());
                    }
                },
                _ => {},
            }
        }

        // Check if shaders have been found
        if !vert_shader.is_some() || !frag_shader.is_some() {
            warn!(desc.label, "Failed to load material resource: missing shaders.");
            return Self {
                label: desc.label.to_string(),
                data: None,
                loaded: false,
                async_loaded,
                temp_data: None,
            };
        }

        Self {
            label: desc.label.to_string(),
            data: None,
            async_loaded,
            loaded: false,
            temp_data: Some(TempMaterialData {
                vert_shader: vert_shader.unwrap(),
                frag_shader: frag_shader.unwrap(),
            }),
        }
    }

    #[tracing::instrument]
    fn sync_load(&mut self, render_instance: &RenderInstance, res_manager: &ResourcesManager) {
        // Check if shaders are set
        if !self.temp_data.is_some() {
            warn!(self.label, "Failed to sync load material resource: missing shaders.");
            return;
        }

        // Check if shaders are loaded
        let vert_shader = match res_manager.get::<ShaderResource>(&self.temp_data.as_ref().unwrap().vert_shader) {
            Some(vert_shader) => vert_shader,
            None => {
                // Try again later
                return;
            }
        };
        let frag_shader = match res_manager.get::<ShaderResource>(&self.temp_data.as_ref().unwrap().frag_shader) {
            Some(frag_shader) => frag_shader,
            None => {
                // Try again later
                return;
            }
        };

        // Create material pipeline and set shaders
        let mut render_pipeline = RenderPipeline::new(format!("Material {}", self.label).as_str());
        let _ = render_pipeline
            .set_shader(&vert_shader.data.as_ref().unwrap().module, ShaderType::Vertex)
            .set_shader(&frag_shader.data.as_ref().unwrap().module, ShaderType::Fragment)
            .set_depth_stencil();

        // Create material data
        self.data = Some(MaterialData {
            pipeline: render_pipeline,
        });

        // Clear temp data
        self.temp_data = None;

        // Set loaded flag
        self.loaded = true;
    }


    // Inherited methods
    fn async_loaded(&self) -> bool { self.async_loaded.flag.lock().unwrap().clone() }
    fn loaded(&self) -> bool { self.loaded }
    fn resource_type() -> ResourceType { ResourceType::Material }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn as_any(&self) -> &dyn Any { self }
}

impl Drop for MaterialResource {
    #[tracing::instrument]
    fn drop(&mut self) {
        info!(self.label, "Unloading material resource.");
    }
}
