use std::{collections::{HashMap, VecDeque}, fmt::Formatter, sync::{Arc, Mutex}};

use wde_logger::{error, trace, debug, info};
use wde_wgpu::RenderInstance;

use crate::{Resource, ResourceType};

/// Maximum number of resources per type
const MAX_RESOURCES_PER_TYPE: usize = 100;

/// The unique identifier of a resource handle
type ResourceHandleIndex = usize;
/// Number of handles pointing to a resource location
type HandleCount = usize;
/// Index pointing to a resource in the resources array
type ResourceArrayIndex = usize;


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

impl std::fmt::Debug for ResourceHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceHandle")
            .field("label", &self.label)
            .field("resource_type", &self.resource_type)
            .field("index", &self.index)
            .finish()
    }
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
    #[tracing::instrument]
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
    #[tracing::instrument]
    fn drop(&mut self) {
        self.manager.lock().unwrap().remove_handle(self.index, self.resource_type);
    }
}
impl Clone for ResourceHandle {
    #[tracing::instrument]
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
    resources: HashMap<ResourceType, Vec<Option<Arc<Mutex<dyn Resource>>>>>,
    /// Pool of available resources indices.
    resources_indices_pool: HashMap<ResourceType, VecDeque<usize>>,
    /// Resources indices that are currently async loading
    resources_async_loading: Vec<(ResourceType, ResourceHandleIndex)>
}

impl std::fmt::Debug for ResourcesManagerInstance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let resources = self.resources.iter().map(|(k, v)| (k, v.len())).collect::<HashMap<_, _>>();

        f.debug_struct("ResourcesManagerInstance")
            .field("handle_index_iterator", &self.handle_index_iterator)
            .field("path_to_index", &self.path_to_index)
            .field("handle_to_res", &self.handle_to_res)
            .field("resources", &resources)
            .field("resources_async_loading", &self.resources_async_loading)
            .finish()
    }
}

impl ResourcesManagerInstance {
    /// Create a new resources manager instance.
    #[tracing::instrument]
    pub fn new() -> Self {
        trace!("Creating resources manager instance.");

        Self {
            handle_index_iterator: 0,
            path_to_index: HashMap::new(),
            handle_to_res: HashMap::new(),

            resources: HashMap::new(),
            resources_indices_pool: HashMap::new(),
            resources_async_loading: Vec::new(),
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
            if self.handle_to_res.contains_key(&index) && self.handle_to_res.get(&index).unwrap().0 > 0 {
                // Return resource index
                return *index;
            }
        }
        
        // Check if resource type exists
        let resource_type = T::resource_type();
        if !self.resources.contains_key(&resource_type) {
            // Create resource type array
            self.resources.insert(resource_type, Vec::from_iter((0..MAX_RESOURCES_PER_TYPE).map(|_| None)));
            // Create resource type indices pool
            self.resources_indices_pool.insert(
                resource_type,
                VecDeque::from_iter((0..MAX_RESOURCES_PER_TYPE).map(|i| i))
            );
        }
        
        // Generate new resource handle index
        let index = self.handle_index_iterator;
        self.handle_index_iterator += 1;
        
        // Add resource to path to resource map
        self.path_to_index.insert(path.to_string(), index);
        
        // Start loading resource
        debug!(path, "Loading resource.");
        let resource = T::new(path);
        
        // Add resource to resources list
        let resources_arr = self.resources.get_mut(&resource_type).unwrap();
        let resource_index = self.resources_indices_pool.get_mut(&resource_type).unwrap().pop_front().unwrap();
        resources_arr[resource_index] = Some(Arc::new(Mutex::new(resource)));
        
        // Add resource to handle to resource map and async loading list
        self.handle_to_res.insert(index, (0, resource_index));
        self.resources_async_loading.push((resource_type, resource_index));

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

            // Remove resource from async loading list
            if let Some(index) = self.resources_async_loading.iter().position(|&r| r == (resource_type, resource_index_g)) {
                self.resources_async_loading.remove(index);
            }

            // Remove resource from resources list
            debug!(resource_index_g, "Unloading resource.");
            let resources_arr = self.resources.get_mut(&resource_type).unwrap();
            resources_arr[resource_index_g] = None;

            // Add resource index to indices pool
            self.resources_indices_pool.get_mut(&resource_type).unwrap().push_back(resource_index_g);
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
///       // Get resource (returns None if resource is not yet loaded)
///       let res = res_manager.get::<DummyResource>(&handle2);
///       (...)
///    } // Drop handle2 -> Resource is still loaded
/// 
///    // Get resource
///    let res = res_manager.get::<DummyResource>(&handle);
///    (...)
/// } // Drop handle -> Resource is unloaded
/// 
/// // Update resources manager (every frame)
/// loop {
///     res_manager.update(&render_instance);
///     (...)
/// }
/// ```
#[derive(Debug)]
pub struct ResourcesManager {
    /// Resources manager instance
    instance: Arc<Mutex<ResourcesManagerInstance>>,
}

impl ResourcesManager {
    /// Create a new resources manager instance.
    #[tracing::instrument]
    pub fn new() -> Self {
        info!("Creating resources manager.");

        Self {
            instance: Arc::new(Mutex::new(ResourcesManagerInstance::new())),
        }
    }

    /// Update the resources manager.
    /// This will check if async loaded resources are loaded, and if so, remove them from the async loading list.
    /// Then, it will run the sync loading and set the loaded flag to true.
    /// In particular, it will transfer the data on the GPU.
    /// This function should be called at the beginning of each frame.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance.
    #[tracing::instrument]
    pub fn update(&mut self, render_instance: &RenderInstance) {
        debug!("Updating resources manager.");
        let mut instance = self.instance.lock().unwrap();

        // Check if async loaded resources are loaded
        let mut should_remove = Vec::new();
        for (resource_type, resource_index) in instance.resources_async_loading.clone() {
            // Get resource
            let resources_arr = instance.resources
                .get_mut(&resource_type).unwrap()
                .get(resource_index as usize).unwrap();

            // Check if resource is not none
            if resources_arr.is_none() {
                continue;
            }
            
            // If resource is loaded, sync load it and remove it from async loading list
            let mut res_ref = resources_arr.as_ref().unwrap().lock().unwrap();
            if res_ref.async_loaded() && !res_ref.loaded() {
                res_ref.sync_load(render_instance);
                should_remove.push((resource_type, resource_index));
            }
        }

        // Remove async loaded resources from async loading list
        for (resource_type, resource_index) in should_remove {
            let index = instance.resources_async_loading.iter().position(|&r| r == (resource_type, resource_index)).unwrap();
            instance.resources_async_loading.remove(index);
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
    pub fn get<T: Resource>(&self, handle: &ResourceHandle) -> Option<&mut T> {
        let mut instance = self.instance.lock().unwrap();

        // Check if handle is valid
        if !instance.handle_to_res.contains_key(&handle.index) {
            error!(handle.index, "Invalid resource handle.");
            return None;
        }

        // Get resource index
        let (_, resource_index) = instance.handle_to_res.get(&handle.index).unwrap().clone();

        // Get resource
        let resources_arr = instance.resources.get_mut(&T::resource_type()).unwrap();
        let resource_unlocked = resources_arr.get_mut(resource_index as usize).unwrap();
        if resource_unlocked.is_none() {
            return None;
        }
        let mut resource_as_dyn = resource_unlocked.as_mut().unwrap().lock().unwrap();
        let resource_as_t = resource_as_dyn.as_any_mut().downcast_mut::<T>().unwrap();
        if !resource_as_t.loaded() {
            return None;
        }
        Some(unsafe { std::mem::transmute::<&mut T, &mut T>(resource_as_t) })
    }

    /// Wait synchronously for a resource to be loaded.
    /// 
    /// # Arguments
    /// 
    /// * `handle` - Handle pointing to the resource location.
    /// * `render_instance` - The render instance.
    #[tracing::instrument]
    pub async fn wait_for(&mut self, handle: &ResourceHandle, render_instance: &RenderInstance) {
        debug!(handle.index, "Waiting synchronously for resource to be loaded.");
        let mut instance = self.instance.lock().unwrap();

        // Check if handle is valid
        if !instance.handle_to_res.contains_key(&handle.index) {
            error!(handle.index, "Invalid resource handle.");
            return;
        }

        // Get resource index
        let (_, resource_index) = instance.handle_to_res.get(&handle.index).unwrap().clone();

        // Get resource
        let resources_arr = instance.resources.get_mut(&handle.resource_type).unwrap();
        let resource_unlocked = resources_arr.get_mut(resource_index as usize).unwrap();
        if resource_unlocked.is_none() {
            error!(handle.index, "Resource is not loaded.");
            return;
        }
        let mut resource_as_dyn = resource_unlocked.as_mut().unwrap().lock().unwrap();

        // Wait for resource to be async loaded
        while !resource_as_dyn.async_loaded() {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }

        // Sync load resource
        if !resource_as_dyn.loaded() {
            resource_as_dyn.sync_load(render_instance);
        }
    }
}

impl Drop for ResourcesManager {
    #[tracing::instrument]
    fn drop(&mut self) {
        info!("Dropping resources manager.");
    }
}
