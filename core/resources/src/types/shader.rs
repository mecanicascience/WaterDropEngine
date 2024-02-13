use std::{any::Any, fmt::Formatter, sync::{Arc, Mutex}};

use tokio::io::AsyncReadExt;
use wde_logger::{debug, error, info};
use wde_wgpu::RenderInstance;

use crate::{LoadedFlag, Resource, ResourceDescription, ResourceType, ResourcesManager};

/// Temporary data to be transferred.
#[derive(Clone, Debug)]
struct TempShaderData {
    pub content: String,
}

/// Resource data.
#[derive(Debug)]
pub struct ShaderData {
    pub module: String,
}

/// Store a shader resource loaded from a shader file.
/// This resource is loaded asynchronously.
/// The data are stored in the `data` field when loaded.
pub struct ShaderResource {
    /// Label of the shader.
    pub label: String,
    /// Path of the shader file.
    pub path: String,
    /// Shader data.
    pub data: Option<ShaderData>,
    /// Loaded state of the shader.
    loaded: bool,

    // Async loading
    async_loaded: LoadedFlag,
    sync_receiver: std::sync::mpsc::Receiver<TempShaderData>
}

impl std::fmt::Debug for ShaderResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderResource")
            .field("label", &self.label)
            .field("loaded", &self.loaded)
            .field("async_loaded", &self.async_loaded)
            .finish()
    }
}

impl Resource for ShaderResource {
    #[tracing::instrument]
    fn new(desc: ResourceDescription) -> Self where Self: Sized {
        info!(desc.label, "Creating shader resource.");

        // Check if resource type is correct
        if desc.resource_type != Self::resource_type() {
            error!(desc.label, "Trying to create a shader resource with a non shader resource description.");
            return Self {
                label: desc.label.to_string(),
                path: desc.source.to_string(),
                data: None,
                loaded: false,
                async_loaded: LoadedFlag { flag: Arc::new(Mutex::new(false)), },
                sync_receiver: std::sync::mpsc::sync_channel(1).1
            };
        }

        // Create sync resources
        let async_loaded = LoadedFlag { flag: Arc::new(Mutex::new(false)), };
        let async_loaded_c = Arc::clone(&async_loaded.flag);
        let (sync_sender, sync_receiver) = std::sync::mpsc::sync_channel(1);
        let path_c = desc.source.to_string();
        
        // Create async task
        let task = async move {
            // File path
            let path_f = std::env::current_exe().unwrap().as_path()
                .parent().unwrap()
                .join(path_c.clone().replace("/", "\\"));

            // Open file
            let file_status = tokio::fs::File::open(path_f).await;
            if file_status.is_err() {
                error!(path_c, "Failed to open shader file.");
                return;
            }

            // Read file
            let mut content_buffer = Vec::new();
            let read_status = file_status.unwrap().read_to_end(&mut content_buffer).await;
            if read_status.is_err() {
                error!(path_c, "Failed to read shader file.");
                return;
            }

            // Set loading to false
            let mut flag = async_loaded_c.lock().unwrap();
            *flag = true;

            // Log that the shader is async loaded
            debug!(path_c, "Shader is async loaded.");

            // Send data
            let data = TempShaderData { content: String::from_utf8(content_buffer).unwrap() };
            sync_sender.send(data).unwrap_or_else(|e| {
                error!("Failed to send shader data : {}.", e);
            });
        };
        tokio::task::spawn(task);

        Self {
            label: desc.label.to_string(),
            path: desc.source.to_string(),
            data: None,
            loaded: false,
            async_loaded,
            sync_receiver
        }
    }

    #[tracing::instrument]
    fn sync_load(&mut self, _render_instance: &RenderInstance, _res_manager: &ResourcesManager) {
        // Check if the model is async loaded
        if !self.async_loaded() {
            error!("Trying to sync load a shader that is not async loaded.");
            return;
        }
        debug!(self.label, "Sync loading shader.");

        // Set data
        self.data = Some(ShaderData {
            module: self.sync_receiver.recv().unwrap_or_else(|e| {
                error!("Failed to receive shader data : {}.", e);
                TempShaderData { content: String::new() }
            }).content
        });

        // Set loaded flag
        self.loaded = true;
    }


    // Inherited methods
    fn async_loaded(&self) -> bool { self.async_loaded.flag.lock().unwrap().clone() }
    fn loaded(&self) -> bool { self.loaded }
    fn resource_type() -> ResourceType { ResourceType::Shader }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn as_any(&self) -> &dyn Any { self }
}

impl Drop for ShaderResource {
    #[tracing::instrument]
    fn drop(&mut self) {
        info!(self.label, "Unloading shader resource.");
    }
}
