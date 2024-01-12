use std::any::Any;

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

    /// As any.
    fn as_any(&self) -> &dyn Any;
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
        Self {
            label: label.to_string(),
            loaded: false,
        }
    }

    /// Check if the resource is loaded.
    /// 
    /// # Returns
    /// 
    /// * `bool` - True if the resource is loaded, false otherwise.
    fn loaded(&self) -> bool {
        self.loaded
    }

    /// As any.
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
