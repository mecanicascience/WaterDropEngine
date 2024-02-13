use std::{any::Any, sync::{Arc, Mutex, RwLock}};

use tracing::error;
use wde_logger::throw;
use wde_wgpu::RenderInstance;

use crate::{MaterialResource, ModelResource, ResourceHandle, ResourcesManager, ShaderResource};

// Struct to hold the resource loading flag
#[derive(Debug)]
pub struct LoadedFlag {
    pub flag: Arc<Mutex<bool>>,
}

/// List of resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// Model resource.
    Model,
    /// Shader resource.
    Shader,
    /// Material resource.
    Material
}

/// String to resource type conversion.
impl From<&str> for ResourceType {
    fn from(s: &str) -> Self {
        match s {
            "MODEL" => ResourceType::Model,
            "SHADER" => ResourceType::Shader,
            "MATERIAL" => ResourceType::Material,
            _ => throw!("Unknown resource type: {}", s)
        }
    }
}

/// Create a resource instance from a type.
/// 
/// # Arguments
/// 
/// * `res_type` - The type of the resource.
/// * `desc` - The description of the resource.
pub fn create_resource_instance(res_type: &ResourceType, desc: ResourceDescription) -> Arc<RwLock<dyn Resource>> {
    match res_type {
        ResourceType::Model => Arc::new(RwLock::new(ModelResource::new(desc))),
        ResourceType::Shader => Arc::new(RwLock::new(ShaderResource::new(desc))),
        ResourceType::Material => Arc::new(RwLock::new(MaterialResource::new(desc)))
    }
}

/// Description of a resource.
#[derive(Debug)]
pub struct ResourceDescription {
    /// The label of the resource.
    pub label: String,
    /// The type of the resource.
    pub resource_type: ResourceType,
    /// The source of the resource.
    pub source: String,
    /// List of dependencies.
    pub dependencies: Vec<ResourceHandle>,
}

/// Unresolved description of a resource.
/// Here, the dependencies are not yet handles.
#[derive(Debug)]
pub struct ResourceDescriptionUnresolved {
    /// The label of the resource.
    pub label: String,
    /// The type of the resource.
    pub resource_type: ResourceType,
    /// The source of the resource.
    pub source: String,
    /// List of dependencies.
    pub dependencies: Vec<Option<String>>,
}


/// Describe a resource.
pub trait Resource: Any {
    /// Create a new resource.
    /// This will start the async loading of the resource.
    /// Please note that the description of the resource needs to be stored to avoid mutex lockouts.
    /// 
    /// # Arguments
    /// 
    /// * `desc` - The description of the resource.
    fn new(desc: ResourceDescription) -> Self where Self: Sized;

    /// Load the sync part of the resource.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    fn sync_load(&mut self, instance: &RenderInstance, res_manager: &ResourcesManager);


    /// Check if the resource is async loaded.
    /// 
    /// # Returns
    /// 
    /// * `bool` - True if the resource is async loaded, false otherwise.
    fn async_loaded(&self) -> bool;

    /// Check if the resource is loaded.
    /// 
    /// # Returns
    /// 
    /// * `bool` - True if the resource is loaded, false otherwise.
    fn loaded(&self) -> bool;

    /// Get the type of the resource.
    ///
    /// 
    /// # Returns
    /// 
    /// * `ResourceType` - The type of the resource.
    fn resource_type() -> ResourceType where Self: Sized;

    /// As any mut.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// As any.
    fn as_any(&self) -> &dyn Any;
}

/// Get the description of a resource from its JSON.
/// 
/// # Arguments
/// 
/// * `path` - Path to the resource.
/// * `resource_json` - JSON value of the resource description.
#[tracing::instrument]
pub fn get_resource_description(path: &str, resource_json: &serde_json::Value) -> Option<ResourceDescriptionUnresolved> {
    // Get label
    let label = match resource_json.get("label") {
        Some(label_json) => {
            match label_json.as_str() {
                Some(label_json) => {
                    label_json.to_string()
                },
                None => {
                    error!(path, "Failed to get resource label while parsing.");
                    path.to_string()
                }
            }
        },
        None => {
            error!(path, "Resource has no label.");
            path.to_string()
        },
    };

    // Get metadata
    let (resource_type, source, dependencies) = match resource_json.get("metadata") {
        Some(metadata) => {
            // Get resource type
            let resource_type = match metadata.get("type") {
                Some(resource_type) => {
                    match resource_type.as_str() {
                        Some(resource_type) => {
                            ResourceType::from(resource_type)
                        },
                        None => {
                            error!(label, path, "Failed to get resource type while parsing.");
                            return None;
                        }
                    }
                },
                None => {
                    error!(label, path, "Resource has no type.");
                    return None;
                }
            };

            // Get source
            let source = match metadata.get("source") {
                Some(source) => {
                    match source.as_str() {
                        Some(source) => {
                            ("res/".to_string() + source).to_string()
                        },
                        None => {
                            error!(label, path, "Failed to get resource source while parsing.");
                            return None;
                        }
                    }
                },
                None => {
                    error!(label, path, "Resource has no source.");
                    return None;
                }
            };

            // Get dependencies paths
            let dependencies: Vec<Option<String>> = match metadata.get("dependencies") {
                Some(dependencies) => {
                    match dependencies.as_array() {
                        Some(dependencies) => {
                            let mut vec: Vec<Option<String>> = vec![None; dependencies.len()];
                            for (i, d) in dependencies.iter().enumerate() {
                                match d.get("path") {
                                    Some(d_path) => {
                                        match d_path.as_str() {
                                            Some(d_path) => {
                                                vec[i] = Some(d_path.to_string());
                                            }
                                            None => {
                                                error!(path, "Failed to get resource dependencies path while parsing.");
                                                return None;
                                            }
                                        };
                                    },
                                    None => { error!(path, "Failed to get resource dependencies name."); }
                                };
                            }
                            vec
                        },
                        None => { error!(path, "Failed to get resource dependencies while parsing."); Vec::new() }
                    }
                },
                None => { error!(path, "Resource has no dependencies."); Vec::new() }
            };

            // Return metadata
            (resource_type, source, dependencies)
        },
        None => {
            error!(path, "Failed to get resource metadata while parsing.");
            return None;
        }
    };

    // Create resource description
    Some(ResourceDescriptionUnresolved {
        label,
        resource_type,
        source,
        dependencies,
    })
}
