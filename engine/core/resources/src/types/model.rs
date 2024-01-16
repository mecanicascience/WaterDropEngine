use std::{any::Any, sync::{Arc, Mutex}};

use wde_logger::{debug, error, trace, info};
use wde_math::Vec3f;
use wde_wgpu::{Vertex, Buffer, RenderInstance, BufferUsage};

use crate::{Resource, ResourceType, LoadedFlag};

/// Bounding box of a model, centered at the origin.
#[derive(Clone, Copy)]
pub struct ModelBoundingBox {
    pub min: Vec3f,
    pub max: Vec3f
}

/// Temporary data to be transferred.
struct TempModelData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub bounding_box: ModelBoundingBox,
}

/// Resource data.
pub struct ModelData {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub vertex_count: u32,
    pub bounding_box: ModelBoundingBox,
}

/// Store a model resource loaded from a model file.
/// This resource is loaded asynchronously.
/// The data are stored in the `data` field when loaded.
pub struct ModelResource {
    /// Label of the model.
    pub label: String,
    /// Model data.
    pub data: Option<ModelData>,
    /// Loaded state of the model.
    loaded: bool,

    // Async loading
    async_loaded: LoadedFlag,
    sync_receiver: std::sync::mpsc::Receiver<TempModelData>,
}

impl Resource for ModelResource {
    /// Create a new model resource.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the model.
    fn new(label: &str) -> Self {
        info!("Creating model resource '{}'.", label);

        // Create sync resources
        let async_loaded = LoadedFlag { flag: Arc::new(Mutex::new(false)), };
        let async_loaded_c = Arc::clone(&async_loaded.flag);
        let (sync_sender, sync_receiver) = std::sync::mpsc::sync_channel(1);
        let path_c = label.to_string();
        
        // Create async task
        tokio::task::spawn(async move {
            let mut vertices: Vec<Vertex> = Vec::new();
            let mut indices: Vec<u32> = Vec::new();

            // File path
            let path_f = std::env::current_exe().unwrap().as_path()
                .parent().unwrap()
                .join("res")
                .join(path_c.clone().replace("/", "\\"));

            // Open file
            trace!("Loading model from file: '{}'.", path_c);
            let load_res = tobj::load_obj(
                    path_f,
                    &tobj::LoadOptions {
                        single_index: true,
                        ..Default::default()
                    }
                );
            if let Err(e) = load_res {
                error!("Failed to load model from file: '{}' : {:?}.", path_c, e);
                return;
            }
            let (models, _) = load_res.unwrap();

            // Bounding box of the model
            let mut bounding_box = ModelBoundingBox {
                min: Vec3f::new(std::f32::MAX, std::f32::MAX, std::f32::MAX),
                max: Vec3f::new(std::f32::MIN, std::f32::MIN, std::f32::MIN),
            };

            // Load models
            for (_, m) in models.iter().enumerate() {
                let mesh = &m.mesh;
                if mesh.positions.len() % 3 != 0 {
                    error!("Mesh positions are not divisible by 3 for model '{}'.", path_c);
                    return;
                }

                // Allocate sizes
                vertices.reserve(mesh.positions.len() / 3);

                // Create vertices
                for vtx in 0..mesh.positions.len() / 3 {
                    let x = mesh.positions[3 * vtx];
                    let y = mesh.positions[3 * vtx + 1];
                    let z = mesh.positions[3 * vtx + 2];

                    // Normals
                    let mut nx = 0.0;
                    let mut ny = 0.0;
                    let mut nz = 0.0;
                    if mesh.normals.len() >= 3 * vtx + 2 {
                        nx = mesh.normals[3 * vtx];
                        ny = mesh.normals[3 * vtx + 1];
                        nz = mesh.normals[3 * vtx + 2];
                    }

                    // UVs
                    let mut u = 0.0;
                    let mut v = 0.0;
                    if mesh.texcoords.len() >= 2 * vtx + 1 {
                        u = mesh.texcoords[2 * vtx];
                        v = mesh.texcoords[2 * vtx + 1];
                    }

                    // Vertex
                    vertices.push(Vertex {
                        position: [x, y, z],
                        normal: [nx, ny, nz],
                        tex_uv: [u, v],
                    });

                    // Update bounding box
                    bounding_box.min.x = bounding_box.min.x.min(x);
                    bounding_box.min.y = bounding_box.min.y.min(y);
                    bounding_box.min.z = bounding_box.min.z.min(z);
                    bounding_box.max.x = bounding_box.max.x.max(x);
                    bounding_box.max.y = bounding_box.max.y.max(y);
                    bounding_box.max.z = bounding_box.max.z.max(z);
                }

                // Push indices
                indices.extend_from_slice(&mesh.indices);
            }

            // Set loading to false
            let mut flag = async_loaded_c.lock().unwrap();
            *flag = true;

            // Log that the model is async loaded
            debug!("Model '{}' is async loaded.", path_c);

            // Send data
            let data = TempModelData {
                vertices,
                indices,
                bounding_box,
            };
            sync_sender.send(data).unwrap_or_else(|e| {
                error!("Failed to send model data : {}", e);
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

    fn sync_load(&mut self, instance: &RenderInstance) {
        // Check if the model is async loaded
        if !self.async_loaded() {
            error!("Trying to sync load a model that is not async loaded.");
            return;
        }
        debug!("Sync loading model '{}'.", self.label);

        // Receive data
        let temp_data = self.sync_receiver.recv().unwrap_or_else(|e| {
            error!("Failed to receive model data : {}", e);
            TempModelData {
                vertices: Vec::new(),
                indices: Vec::new(),
                bounding_box: ModelBoundingBox {
                    min: Vec3f::new(0.0, 0.0, 0.0),
                    max: Vec3f::new(0.0, 0.0, 0.0),
                },
            }
        });

        // Create vertex buffer
        let vertex_buffer = Buffer::new(
            &instance,
            format!("'{}' Vertex", self.label).as_str(),
            std::mem::size_of::<Vertex>() * temp_data.vertices.len(),
            BufferUsage::VERTEX,
            Some(bytemuck::cast_slice(&temp_data.vertices)));

        // Create index buffer
        let index_buffer = Buffer::new(
            &instance,
            format!("'{}' Index", self.label).as_str(),
            std::mem::size_of::<u32>() * temp_data.indices.len(),
            BufferUsage::INDEX,
            Some(bytemuck::cast_slice(&temp_data.indices)));

        // Set data
        self.data = Some(ModelData {
            bounding_box: temp_data.bounding_box,
            vertex_buffer,
            index_buffer,
            index_count: temp_data.indices.len() as u32,
            vertex_count: temp_data.vertices.len() as u32,
        });

        // Set loaded flag
        self.loaded = true;
    }


    // Inherited methods
    fn async_loaded(&self) -> bool { self.async_loaded.flag.lock().unwrap().clone() }
    fn loaded(&self) -> bool { self.loaded }
    fn resource_type() -> ResourceType { ResourceType::Model }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Drop for ModelResource {
    fn drop(&mut self) {
        info!("Unloading model resource '{}'.", self.label);
    }
}
