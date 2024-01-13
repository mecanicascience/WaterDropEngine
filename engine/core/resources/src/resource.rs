use std::{any::Any, sync::{Arc, Mutex}};

use wde_wgpu::RenderInstance;

// Struct to hold the resource loading flag
pub struct LoadedFlag {
    pub flag: Arc<Mutex<bool>>,
}

/// List of resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// Model resource.
    Model,
}


/// Describe a resource.
pub trait Resource: Any {
    /// Create a new resource.
    /// This will start the async loading of the resource.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the resource.
    fn new(label: &str) -> Self where Self: Sized;

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
