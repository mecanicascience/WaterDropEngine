use std::{any::TypeId, collections::HashMap};

use wde_logger::{info, error, debug};

use crate::{EntityIndex, ComponentIndex};


/// An interface for a list of components.
/// This is used to store multiple components of different types in a single list.
pub trait IComponentArray {
    fn entity_destroyed(&mut self, entity: EntityIndex);
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// A list of components of a specific type `T`.
#[derive(Debug)]
pub struct ComponentArray<T: 'static> {
    /// List of components in this array
    pub components: Vec<T>,
    /// Number of components in this array
    pub size: usize,

    /// Map of entities indices to their index in the components list
    pub entity_to_index_map: HashMap<EntityIndex, usize>,
    /// Map of component indices to their corresponding entity indices
    pub index_to_entity_map: HashMap<usize, EntityIndex>,
}

impl<T: 'static> IComponentArray for ComponentArray<T> {
    fn entity_destroyed(&mut self, entity: EntityIndex) {
        if let Some(_) = self.entity_to_index_map.get(&entity) {
            self.remove_data(entity);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

impl<T: 'static> ComponentArray<T> {
    /// Creates a new component array.
    #[tracing::instrument]
    pub fn new() -> Self {
        debug!(component_type=std::any::type_name::<T>(), "Creating new component array.");

        Self {
            components: Vec::new(),
            size: 0,
            entity_to_index_map: HashMap::new(),
            index_to_entity_map: HashMap::new(),
        }
    }

    /// Adds a new component to the list corresponding to the given entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity to add the component to.
    /// * `component` - The component to add.
    pub fn insert_data(&mut self, entity: EntityIndex, component: T) {
        if self.entity_to_index_map.contains_key(&entity) {
            error!(component_type=std::any::type_name::<T>(), entity, "Component already exists for entity.");
            return;
        }

        // Add the component as the last element in the list and update the maps
        let index = self.size;
        self.entity_to_index_map.insert(entity, index);
        self.index_to_entity_map.insert(index, entity);
        self.components.push(component);
        self.size += 1;
    }

    /// Removes the component corresponding to the given entity
    /// by swapping it with the last element in the list.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity to remove the component from.
    pub fn remove_data(&mut self, entity: EntityIndex) {
        let to_remove_index = match self.entity_to_index_map.get(&entity) {
            Some(index) => *index,
            None => {
                error!(component_type=std::any::type_name::<T>(), entity, "Component does not exist for entity.");
                return;
            }
        };

        // Copy the last element in the list to the index of the removed component to avoid gaps
        self.components.swap_remove(to_remove_index);

        // Update the index of the moved component
        let index_of_last = self.size - 1;
        if let Some(moved_entity) = self.index_to_entity_map.get(&index_of_last) {
            self.entity_to_index_map.insert(*moved_entity, to_remove_index);
            self.index_to_entity_map.insert(to_remove_index, *moved_entity);
        }

        // Remove the component from the maps
        self.entity_to_index_map.remove(&entity);
        self.index_to_entity_map.remove(&to_remove_index);

        self.size -= 1;
    }

    /// Gets an immutable reference to the component corresponding to the given entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity to get the component for.
    /// 
    /// # Returns
    /// 
    /// * `Some(&T)` - If the component exists.
    /// * `None` - If the component does not exist.
    pub fn get_data(&self, entity: EntityIndex) -> Option<&T> {
        match self.entity_to_index_map.get(&entity) {
            Some(index) => Some(&self.components[*index]),
            None => None
        }
    }

    /// Sets the value of the component corresponding to the given entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity to set the component for.
    /// * `component` - The component to set.
    pub fn set_data(&mut self, entity: EntityIndex, component: T) {
        match self.entity_to_index_map.get(&entity) {
            Some(index) => {
                self.components[*index] = component;
            },
            None => {
                error!(component_type=std::any::type_name::<T>(), entity, "Component does not exist for entity.");
            }
        }
    }

    /// Gets the list of entities with components of this type.
    /// 
    /// # Returns
    /// 
    /// * `Vec<EntityIndex>` - The list of entities of this type.
    pub fn get_entities(&self) -> Vec<EntityIndex> {
        self.entity_to_index_map.keys().cloned().collect()
    }
}




/// The component manager which stores all component arrays.
/// 
/// # Example
/// 
/// ```
/// // Create a new component manager
/// let mut component_manager = ComponentManager::new();
/// 
/// // Register a new component
/// component_manager.register_component::<LabelComponent>();
/// 
/// // Add a new component to an entity
/// component_manager.add_component::<LabelComponent>(entity, LabelComponent { label : "Hello world" });
/// 
/// // Get the component of an entity
/// let label = component_manager.get_component::<LabelComponent>(entity);
/// 
/// // Remove the component of an entity
/// component_manager.remove_component::<LabelComponent>(entity);
/// 
/// // Modify the component of an entity
/// component_manager.set_component::<LabelComponent>(entity, LabelComponent { label : "New name !!" });
/// 
/// // Get all entities with a given component
/// let entities = component_manager.get_entities_with_component::<LabelComponent>();
/// ```
pub struct ComponentManager {
    /// List of component types.
    pub component_types_list: Vec<TypeId>,
    /// Number of component types.
    pub component_type_count: ComponentIndex,
    /// Map of component types id to their corresponding component indices.
    pub component_types: HashMap<TypeId, ComponentIndex>,
    /// Name of the component types.
    pub component_names: Vec<String>,

    /// Map of component types id to their corresponding component arrays.
    /// This is used to store multiple components of different types in a single list.
    pub components: HashMap<TypeId, Box<dyn IComponentArray>>,
}

impl std::fmt::Debug for ComponentManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let components = self.components.iter()
            .map(|(k, v)| (k, v.as_any())).collect::<Vec<_>>();

        f.debug_struct("ComponentManager")
            .field("component_types_list", &self.component_types_list)
            .field("component_type_count", &self.component_type_count)
            .field("component_types", &self.component_types)
            .field("components", &components)
            .finish()
    }
}

impl ComponentManager {
    /// Creates a new component manager.
    #[tracing::instrument]
    pub fn new() -> Self {
        info!("Creating a new component manager.");

        Self {
            component_types_list: Vec::new(),
            component_type_count: 0,
            component_types: HashMap::new(),
            component_names: Vec::new(),
            components: HashMap::new()
        }
    }

    /// Registers a new component type with the component manager.
    pub fn register_component<T: 'static>(&mut self, name: &str) {
        info!(component_type=std::any::type_name::<T>(), "Registering new component.");

        self.component_types_list.push(TypeId::of::<T>());
        self.component_types.insert(TypeId::of::<T>(), self.component_type_count);
        self.components.insert(TypeId::of::<T>(), Box::new(ComponentArray::<T>::new()));
        self.component_names.push(name.to_string());
        self.component_type_count += 1;
    }

    /// Adds a new component to the list corresponding to the given entity.
    ///
    /// # Arguments
    /// 
    /// * `entity` - The entity index to add the component to.
    /// * `component` - The component instance to add.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the component was successfully added.
    /// * `Err(())` - If the component could not be added.
    pub fn add_component<T: 'static>(&mut self, entity: EntityIndex, component: T) -> Result<(), ()> {
        match self.components.get_mut(&std::any::TypeId::of::<T>()) {
            Some(component_array) => {
                component_array
                    .as_any_mut()
                    .downcast_mut::<ComponentArray<T>>().unwrap()
                    .insert_data(entity, component);
                Ok(())
            },
            None => {
                error!(component_type=std::any::type_name::<T>(), "Component type does not exist.");
                Err(())
            }
        }
    }

    /// Removes the component corresponding to the given entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity index to remove the component from.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the component was successfully removed.
    /// * `Err(())` - If the component could not be removed as it does not exist.
    pub fn remove_component<T: 'static>(&mut self, entity: EntityIndex) -> Result<(), ()> {
        match self.components.get_mut(&std::any::TypeId::of::<T>()) {
            Some(component_array) => {
                component_array
                    .as_any_mut()
                    .downcast_mut::<ComponentArray<T>>().unwrap()
                    .remove_data(entity);
                Ok(())
            },
            None => {
                error!(component_type=std::any::type_name::<T>(), "Component type does not exist.");
                Err(())
            }
        }
    }

    /// Gets a reference to the component corresponding to the given entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity to get the component for.
    /// 
    /// # Returns
    /// 
    /// * `Some(&T)` - If the component exists.
    /// * `None` - If the component does not exist.
    pub fn get_component<T: 'static>(&self, entity: EntityIndex) -> Option<&T> {
        match self.components.get(&std::any::TypeId::of::<T>()) {
            Some(component_array) => {
                component_array
                    .as_any()
                    .downcast_ref::<ComponentArray<T>>().unwrap()
                    .get_data(entity)
            },
            None => None
        }
    }

    /// Sets the value of the component corresponding to the given entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity to set the component for.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` - If the component was successfully set.
    /// * `Err(())` - If the component could not be set as it does not exist.
    pub fn set_component<T: 'static>(&mut self, entity: EntityIndex, component: T) -> Result<(), ()> {
        match self.components.get_mut(&std::any::TypeId::of::<T>()) {
            Some(component_array) => {
                component_array
                    .as_any_mut()
                    .downcast_mut::<ComponentArray<T>>().unwrap()
                    .set_data(entity, component);
                Ok(())
            },
            None => {
                error!(component_type=std::any::type_name::<T>(), "Component type does not exist.");
                Err(())
            }
        }
    }

    /// Gets all of the entity for a given component.
    /// 
    /// # Returns
    /// 
    /// * `Vec<EntityIndex>` - The list of entities with the given component.
    /// If the component type does not exist, an empty list is returned.
    pub fn get_entities_with_component<T: 'static>(&self) -> Vec<EntityIndex> {
        match self.components.get(&std::any::TypeId::of::<T>()) {
            Some(component_array) => {
                component_array
                    .as_any()
                    .downcast_ref::<ComponentArray<T>>().unwrap()
                    .get_entities()
            },
            None => {
                error!(component_type=std::any::type_name::<T>(), "Component type does not exist.");
                Vec::new()
            }
        }
    }

    /// Gets the type of the component.
    /// 
    /// # Arguments
    /// 
    /// * `T` - The type of the component.
    /// 
    /// # Returns
    /// 
    /// * `Option<ComponentIndex>` - The index of the component type.
    pub fn get_component_type<T: 'static>(&self) -> Option<ComponentIndex> {
        match self.component_types.get(&std::any::TypeId::of::<T>()) {
            Some(component_type) => Some(*component_type),
            None => {
                error!(component_type=std::any::type_name::<T>(), "Component type does not exist.");
                None
            }
        }
    }

    /// Notifies the component manager that the given entity has been destroyed.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity that has been destroyed
    pub fn entity_destroyed(&mut self, entity: EntityIndex) {
        for component_array in self.components.values_mut() {
            component_array.entity_destroyed(entity);
        }
    }
}

