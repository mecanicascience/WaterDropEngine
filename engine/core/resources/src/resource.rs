use std::{any::Any, sync::{Arc, Mutex}};

use wde_logger::throw;
use wde_wgpu::RenderInstance;

use crate::ResourceHandle;

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
    pub dependencies: Vec<Option<ResourceHandle>>,
}


/// Describe a resource.
pub trait Resource: Any {
    /// Create a new resource.
    /// This will start the async loading of the resource.
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
    fn sync_load(&mut self, instance: &RenderInstance);


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

    /// As any.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
