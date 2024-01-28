use std::{any::Any, sync::{Arc, Mutex}};

use wde_logger::info;
use wde_wgpu::{RenderInstance, RenderPipeline};

use crate::{Resource, ResourceType, LoadedFlag};

/// Temporary data to be transferred.
#[derive(Clone, Debug)]
struct TempMaterialData {
    pub content: String,
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

    // Async loading
    async_loaded: LoadedFlag,
}

impl Resource for MaterialResource {
    /// Create a new material resource.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the material.
    #[tracing::instrument]
    fn new(label: &str) -> Self {
        info!(label, "Creating material resource.");

        // Create async loaded flag
        let async_loaded = LoadedFlag { flag: Arc::new(Mutex::new(true)), };

        Self {
            label: label.to_string(),
            data: None,
            async_loaded,
            loaded: false,
        }
    }

    #[tracing::instrument]
    fn sync_load(&mut self, render_instance: &RenderInstance) {
        // // Create shaders
        // let vertex_shader_handle = res_manager.load::<ShaderResource>("shaders/vertex.wgsl");
        // let fragment_shader_handle = res_manager.load::<ShaderResource>("shaders/frag.wgsl");

        // // Wait for shaders to load
        // res_manager.wait_for(&vertex_shader_handle, &render_instance).await;
        // res_manager.wait_for(&fragment_shader_handle, &render_instance).await;

        // // Create camera bind group layout
        // let camera_buffer_bind_group_layout = camera_buffer.create_bind_group_layout(
        //     &render_instance,
        //     wde_wgpu::BufferBindingType::Uniform,
        //     ShaderStages::VERTEX).await;

        // // Create default render pipeline
        // let mut render_pipeline = RenderPipeline::new("Main Render");
        // let _ = render_pipeline
        //     .set_shader(&res_manager.get::<ShaderResource>(&vertex_shader_handle).unwrap().data.as_ref().unwrap().module, ShaderType::Vertex)
        //     .set_shader(&res_manager.get::<ShaderResource>(&fragment_shader_handle).unwrap().data.as_ref().unwrap().module, ShaderType::Fragment)
        //     .add_bind_group(camera_buffer_bind_group_layout)
        //     .add_bind_group(objects_bind_group_layout)
        //     .init(&render_instance).await;

        // Set loaded flag
        self.loaded = true;
    }


    // Inherited methods
    fn async_loaded(&self) -> bool { self.async_loaded.flag.lock().unwrap().clone() }
    fn loaded(&self) -> bool { self.loaded }
    fn resource_type() -> ResourceType { ResourceType::Material }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Drop for MaterialResource {
    #[tracing::instrument]
    fn drop(&mut self) {
        info!(self.label, "Unloading material resource.");
    }
}
