use std::{sync::{Arc, Mutex}, collections::HashMap};

use wde_logger::{error, trace};

use crate::{Resource, ResourceType};

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
            resource_type,
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
    
    /// Resources list. One array per resource type.
    resources: HashMap<ResourceType, Vec<Arc<Mutex<dyn Resource>>>>
}

impl ResourcesManagerInstance {
    /// Create a new resources manager instance.
    /// Note: This will create an array for each resource type.
    pub fn new() -> Self {
        Self {
            handle_index_iterator: 0,
            path_to_index: HashMap::new(),
            handle_to_res: HashMap::new(),

            resources: HashMap::from([
                (ResourceType::Dummy, Vec::new()),
            ]),
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
    fn load<T: Resource>(&mut self, path: &str) -> ResourceHandleIndex {
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
        trace!("Loading resource with path {}.", path);
        let resource = T::new(path);

        // Add resource to resources list
        let resource_type = T::resource_type();
        let resources_arr = self.resources.get_mut(&resource_type).unwrap();
        resources_arr.push(Arc::new(Mutex::new(resource)));

        // Add resource to handle to resource map
        self.handle_to_res.insert(index, (0, resources_arr.len() as ResourceArrayIndex - 1));

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
            trace!("Unloading resource with index {}. If the program crashes here, you probably dropped the handle too soon. Please use drop(handle) manually.", resource_index_g);
            let arr = self.resources.get_mut(&resource_type).unwrap();
            arr.remove(resource_index_g as usize);
        }
    }
}




/// Resources manager instance.
/// Stores all the resources loaded by the engine, and their handles.
/// Each resource handle is a reference counted pointer to a resource location.
/// When all of the handles pointing to a resource location are dropped, the resource is unloaded.
/// 
/// # Example
/// 
/// ```
/// // Create a new resources manager instance
/// let mut res_manager = ResourcesManager::new();
/// 
/// // Load resource
/// {
///    let handle = res_manager.load::<DummyResource>("test");
/// 
///    {
///       // Clone handle
///       let handle2 = handle.clone();
/// 
///       // Get resource
///       let res = res_manager.get::<DummyResource>(&handle2);
///       (...)
///    } // Drop handle2 -> Resource is still loaded
/// 
///    // Get resource
///    let res = res_manager.get::<DummyResource>(&handle);
///    (...)
/// } // Drop handle -> Resource is unloaded
/// ```
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
    /// 
    /// # Returns
    /// 
    /// * `ResourceHandle` - Handle pointing to the resource location.
    /// When all of the handles pointing to a resource location are dropped, the resource is unloaded.
    pub fn load<T: Resource>(&mut self, path: &str) -> ResourceHandle {
        // Get resource type
        let resource_type = T::resource_type();

        // Start loading resource
        let index: ResourceHandleIndex = self.instance.lock().unwrap().load::<T>(path);

        // Create resource handle and return it
        ResourceHandle::new(path, resource_type, index, self.instance.clone())
    }

    /// Get a resource from a resource handle.
    /// 
    /// # Arguments
    /// 
    /// * `handle` - Handle pointing to the resource location.
    /// 
    /// # Returns
    /// 
    /// * `Option<&T>` - The resource if it exists, None otherwise.
    pub fn get<T: Resource>(&mut self, handle: ResourceHandle) -> Option<&mut T> {
        let mut instance = self.instance.lock().unwrap();

        // Check if handle is valid
        if !instance.handle_to_res.contains_key(&handle.index) {
            error!("Invalid resource handle.");
            return None;
        }

        // Get resource index
        let (_, resource_index) = instance.handle_to_res.get(&handle.index).unwrap().clone();

        // Get resource
        let resources_arr = instance.resources.get_mut(&T::resource_type()).unwrap();
        let mut resource_as_dyn = resources_arr.get_mut(resource_index as usize).unwrap().lock().unwrap();
        let resource_as_t = resource_as_dyn.as_any_mut().downcast_mut::<T>().unwrap();
        if !resource_as_t.loaded() {
            return None;
        }
        Some(unsafe { std::mem::transmute::<&mut T, &mut T>(resource_as_t) })
    }
}
