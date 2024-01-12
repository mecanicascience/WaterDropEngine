use std::{sync::{Arc, Mutex}, collections::HashMap};

use wde_logger::error;

use crate::{DummyResource, Resource, ResourceType};

/// The unique identifier of a resource handle
type ResourceHandleIndex = u32;
/// Number of handles pointing to a resource location
type HandleCount = u32;
/// Index pointing to a resource in the resources array
type ResourceArrayIndex = u32;


/// Represents a handle pointing to a resource location.
/// When all of the handles pointing to a resource location are dropped, the resource is unloaded.
pub struct ResourceHandle {
    /// Label of the resource
    pub label: String,
    /// Type of the resource
    pub resource_type: ResourceType,
    /// Index of the resource handle
    index: ResourceHandleIndex,
    /// Resources manager instance
    manager: Arc<Mutex<ResourcesManagerInstance>>,
}

impl ResourceHandle {
    /// Create a new resource handle.
    /// 
    /// # Arguments
    /// 
    /// * `label` - Label of the resource.
    /// * `resource_type` - Type of the resource.
    /// * `index` - Index of the resource handle.
    /// * `manager` - Resources manager instance.
    fn new(label: &str, resource_type: ResourceType, index: ResourceHandleIndex, manager: Arc<Mutex<ResourcesManagerInstance>>) -> Self {
        let m = manager.clone();
        
        // Add handle to resource location
        m.lock().unwrap().add_handle(index);

        // Return resource handle
        Self {
            label: label.to_string(),
            resource_type: resource_type,
            index,
            manager: m,
        }
    }
}
impl Drop for ResourceHandle {
    fn drop(&mut self) {
        self.manager.lock().unwrap().remove_handle(self.index, self.resource_type);
    }
}
impl Clone for ResourceHandle {
    fn clone(&self) -> Self {
        self.manager.lock().unwrap().add_handle(self.index);
        Self {
            label: self.label.clone(),
            resource_type: self.resource_type,
            index: self.index,
            manager: self.manager.clone(),
        }
    }
}


/// Resources manager instance.
/// Stores all the resources loaded by the engine, and their handles.
struct ResourcesManagerInstance {
    /// Resources handle index iterator
    handle_index_iterator: ResourceHandleIndex,

    /// Map from resources path to resources index
    path_to_index: HashMap<String, ResourceHandleIndex>,
    /// Map from resources index to (handle count, resource array index)
    handle_to_res: HashMap<ResourceHandleIndex, (HandleCount, ResourceArrayIndex)>,
    
    /// Resources list
    resources_dummy: Vec<Arc<Mutex<DummyResource>>>,
}

impl ResourcesManagerInstance {
    /// Create a new resources manager instance.
    pub fn new() -> Self {
        Self {
            handle_index_iterator: 0,
            path_to_index: HashMap::new(),
            handle_to_res: HashMap::new(),

            resources_dummy: Vec::new(),
        }
    }

    /// Load a resource from a path.
    /// If the resource is already loaded, returns the resource index.
    /// If the resource is not loaded, start the loading of the texture and returns the resource index.
    /// Note: This will not add a handle to the resource.
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the resource.
    fn load(&mut self, path: &str, resource_type: ResourceType) -> ResourceHandleIndex {
        // Check if resource is already loaded
        if let Some(index) = self.path_to_index.get(path) {
            // Return resource index
            return *index;
        }

        // Generate new resource handle index
        let index = self.handle_index_iterator;
        self.handle_index_iterator += 1;

        // Add resource to path to resource map
        self.path_to_index.insert(path.to_string(), index);

        // Start loading resource
        let resource_array_index = match resource_type {
            ResourceType::Dummy => {
                // Create dummy resource
                let resource = DummyResource::new(path);

                // Add resource to resources list
                self.resources_dummy.push(Arc::new(Mutex::new(resource)));

                // Return resource index
                self.resources_dummy.len() as ResourceArrayIndex - 1
            },
        };

        // Add resource to handle to resource map
        self.handle_to_res.insert(index, (0, resource_array_index));

        // Return resource index
        index
    }

    /// Add a handle pointing to a resource location.
    fn add_handle(&mut self, handle: ResourceHandleIndex) {
        // Add handle to handle count
        let (handle_count, _) = self.handle_to_res.get_mut(&handle).unwrap();
        *handle_count += 1;
    }

    /// Remove a handle pointing to a resource location.
    /// When no more handles are pointing to this resource, it is unloaded.
    fn remove_handle(&mut self, handle: ResourceHandleIndex, resource_type: ResourceType) {
        // Remove handle from handle count
        let (handle_count_g, resource_index_g ) = {
            let (handle_count, resource_index) = self.handle_to_res.get_mut(&handle).unwrap();
            *handle_count -= 1;
            (*handle_count, *resource_index)
        };

        // If there is no more handle pointing to the resource location, unload the resource
        if handle_count_g <= 0 {
            // Remove resource from handle to resource map
            self.handle_to_res.remove(&handle);

            // Remove resource from resources list
            match resource_type {
                ResourceType::Dummy => {
                    self.resources_dummy.remove(resource_index_g as usize);
                },
            }
        }
    }
}




/// Resources manager instance.
/// Stores all the resources loaded by the engine, and their handles.
/// Each resource handle is a reference counted pointer to a resource location.
/// When all of the handles pointing to a resource location are dropped, the resource is unloaded.
pub struct ResourcesManager {
    /// Resources manager instance
    instance: Arc<Mutex<ResourcesManagerInstance>>,
}

impl ResourcesManager {
    /// Create a new resources manager instance.
    pub fn new() -> Self {
        Self {
            instance: Arc::new(Mutex::new(ResourcesManagerInstance::new())),
        }
    }

    /// Load a resource from a path.
    /// If the resource is already loaded, returns the resource index.
    /// If the resource is not loaded, start the loading of the texture and returns the resource index.
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the resource.
    /// * `resource_type` - Type of the resource.
    /// 
    /// # Returns
    /// 
    /// * `ResourceHandle` - Handle pointing to the resource location.
    /// When all of the handles pointing to a resource location are dropped, the resource is unloaded.
    pub fn load(&mut self, path: &str, resource_type: ResourceType) -> ResourceHandle {
        // Start loading resource
        let index: ResourceHandleIndex = self.instance.lock().unwrap().load(path, resource_type);

        // Create resource handle and return it
        ResourceHandle::new(path, resource_type, index, self.instance.clone())
    }

    /// Get a resource from a resource handle.
    pub fn get_dummy(&self, handle: &ResourceHandle) -> Option<Arc<Mutex<DummyResource>>> {
        let instance = self.instance.lock().unwrap();

        // Check if handle is valid
        if !instance.handle_to_res.contains_key(&handle.index) {
            error!("Invalid resource handle.");
            return None;
        }

        // Get resource index
        let (_, resource_index) = instance.handle_to_res.get(&handle.index).unwrap();

        // Get resource
        match handle.resource_type {
            ResourceType::Dummy => {
                let resource = instance.resources_dummy[*resource_index as usize].clone();

                return Some(resource);
            },
        }
    }
}
