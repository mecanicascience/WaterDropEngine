use std::any::Any;

use wde_logger::info;

use crate::{Resource, ResourceType};

/// Dummy resource.
pub struct ModelResource {
    /// Label of the model.
    pub label: String,
    /// Loaded state of the model.
    loaded: bool,
}

impl Resource for ModelResource {
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

impl Drop for ModelResource {
    fn drop(&mut self) {
        info!("Dropping dummy resource with label : {}", self.label);
    }
}