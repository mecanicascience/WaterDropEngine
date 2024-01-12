use std::any::Any;

use wde_logger::info;

/// List of resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// Dummy resource.
    Dummy,
}


/// Describe a resource.
pub trait Resource: Any {
    /// Create a new resource.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the resource.
    fn new(label: &str) -> Self where Self: Sized;

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



/// Dummy resource.
pub struct DummyResource {
    /// Label of the resource.
    pub label: String,
    /// Loaded state of the resource.
    loaded: bool,
}

impl Resource for DummyResource {
    /// Create a new dummy resource.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the resource.
    fn new(label: &str) -> Self {
        info!("Creating dummy resource with label : {}", label);

        Self {
            label: label.to_string(),
            loaded: true,
        }
    }

    fn loaded(&self) -> bool {
        self.loaded
    }

    fn resource_type() -> ResourceType {
        ResourceType::Dummy
    }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl Drop for DummyResource {
    fn drop(&mut self) {
        info!("Dropping dummy resource with label : {}", self.label);
    }
}
