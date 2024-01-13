use std::{any::Any, sync::{Arc, Mutex}};

use tokio::io::AsyncReadExt;
use wde_logger::{debug, error, info};
use wde_wgpu::RenderInstance;

use crate::{Resource, ResourceType, LoadedFlag};

/// Temporary data to be transferred.
struct TempShaderData {
    pub content: String,
}

/// Resource data.
pub struct ShaderData {
    pub module: String,
}

/// Store a shader resource loaded from a shader file.
/// This resource is loaded asynchronously.
/// The data are stored in the `data` field when loaded.
pub struct ShaderResource {
    /// Label of the shader.
    pub label: String,
    /// Shader data.
    pub data: Option<ShaderData>,
    /// Loaded state of the shader.
    loaded: bool,

    // Async loading
    async_loaded: LoadedFlag,
    sync_receiver: std::sync::mpsc::Receiver<TempShaderData>
}

impl Resource for ShaderResource {
    /// Create a new shader resource.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the shader.
    fn new(label: &str) -> Self {
        info!("Creating shader resource '{}'.", label);

        // Create sync resources
        let async_loaded = LoadedFlag { flag: Arc::new(Mutex::new(false)), };
        let async_loaded_c = Arc::clone(&async_loaded.flag);
        let (sync_sender, sync_receiver) = std::sync::mpsc::sync_channel(1);
        let path_c = label.to_string();
        
        // Create async task
        tokio::task::spawn(async move {
            // File path
            let path_f = std::env::current_exe().unwrap().as_path()
                .parent().unwrap()
                .join("res")
                .join(path_c.clone().replace("/", "\\"));

            // Open file
            let file_status = tokio::fs::File::open(path_f).await;
            if file_status.is_err() {
                error!("Failed to open shader file '{}'.", path_c);
                return;
            }

            // Read file
            let mut content_buffer = Vec::new();
            let read_status = file_status.unwrap().read_to_end(&mut content_buffer).await;
            if read_status.is_err() {
                error!("Failed to read shader file '{}'.", path_c);
                return;
            }

            // Set loading to false
            let mut flag = async_loaded_c.lock().unwrap();
            *flag = true;

            // Log that the shader is async loaded
            debug!("Shader '{}' is async loaded.", path_c);

            // Send data
            let data = TempShaderData { content: String::from_utf8(content_buffer).unwrap() };
            sync_sender.send(data).unwrap_or_else(|e| {
                error!("Failed to send shader data : {}", e);
            });
        });

        Self {
            label: label.to_string(),
            data: None,
            loaded: false,
            async_loaded,
            sync_receiver
        }
    }

    fn sync_load(&mut self, _: &RenderInstance) {
        // Check if the model is async loaded
        if !self.async_loaded() {
            error!("Trying to sync load a shader that is not async loaded.");
            return;
        }
        debug!("Sync loading shader '{}'.", self.label);

        // Set data
        self.data = Some(ShaderData {
            module: self.sync_receiver.recv().unwrap_or_else(|e| {
                error!("Failed to receive shader data : {}", e);
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
}

impl Drop for ShaderResource {
    fn drop(&mut self) {
        info!("Unloading shader resource '{}'.", self.label);
    }
}