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
