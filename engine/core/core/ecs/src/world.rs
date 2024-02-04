use wde_logger::{info, error};

use crate::{EntityManager, ComponentManager};

/// Define the type of a component signature.
/// This is a bitset of all the components an entity has.
pub type ComponentSignature = bit_vec::BitVec;
/// Create a new empty signature.
pub fn empty_signature() -> ComponentSignature {
    ComponentSignature::from_elem(MAX_COMPONENTS, false)
}

/// Type of an entity.
pub type EntityIndex = usize;
/// Type of a component.
pub type ComponentIndex = usize;

/// Maximum number of entities in the manager (max for 1 manager ~2_000_000).
pub const MAX_ENTITIES: usize = 100_000;
/// Maximum number of components that can be created.
pub const MAX_COMPONENTS: usize = 32;



/// An interface that provides interaction with the entity, component and system managers.
/// A world contains all the entities, components and systems of an instance of the game.
/// 
/// # Example
/// 
/// ```
/// // Create a new world
/// let mut world = World::new();
/// world.register_component::<LabelComponent>(); // Register the label component
/// 
/// // Create a new entity
/// let entity = world.create_entity();
/// entity
///     .add_component::<LabelComponent>(LabelComponent { label : "Hello world" })  // Add a label to the entity
///     .set_component::<LabelComponent>(LabelComponent { label : "New name !!" }); // Change the label of the entity
/// let label = entity.get_component::<LabelComponent>();   // Get the label of the entity
/// entity.remove_component::<LabelComponent>();            // Remove the label from the entity
/// 
/// // Remove the entity from the world
/// world.destroy_entity(entity);
/// ```
pub struct World {
    // List of managers.
    pub entity_manager: EntityManager,
    pub component_manager: ComponentManager,
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("entity_manager", &self.entity_manager)
            .field("component_manager", &self.component_manager)
            .finish()
    }
}

impl World {
    /// Creates a new world.
    #[tracing::instrument]
    pub fn new() -> Self {
        info!("Creating a new world.");

        Self {
            entity_manager: EntityManager::new(),
            component_manager: ComponentManager::new(),
        }
    }


    // ======================================
    // ============== ENTITIES ==============
    // ======================================

    /// Creates a new entity in the world and returns its entity id.
    /// 
    /// # Returns
    /// 
    /// * `Some(EntityIndex)` - The entity index of the created entity.
    /// * `None` - If no more entities are available.
    pub fn create_entity(&mut self) -> Option<EntityIndex> {
        self.entity_manager.create_entity()
    }

    /// Destroys a given entity from the world.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity index to destroy.
    /// 
    /// # Returns
    /// 
    /// * `&mut Self` - The world.
    pub fn destroy_entity(&mut self, entity: EntityIndex) -> &mut Self {
        // Destroy the entity
        self.entity_manager.destroy_entity(entity);
        self.component_manager.entity_destroyed(entity);
        self
    }





    // ======================================
    // ============== COMPONENTS ==============
    // ======================================

    /// Registers a new component type to the world.
    /// 
    /// # Returns
    /// 
    /// * `&mut Self` - The world.
    #[tracing::instrument]
    pub fn register_component<T: 'static>(&mut self) -> &mut Self {
        self.component_manager.register_component::<T>();
        self
    }

    /// Adds a given component to an entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity index to add the component to.
    /// 
    /// # Returns
    /// 
    /// * `&mut Self` - The world.
    /// * `None` - If the component type is not registered.
    pub fn add_component<T: 'static + std::fmt::Debug>(&mut self, entity: EntityIndex, component: T) -> Option<&mut Self> {
        // Add the component for the entity
        let ans = self.component_manager.add_component::<T>(entity, component);
        if ans.is_err() {
            return None;
        }

        // Add the component to the entities signature
        let mut signature = self.entity_manager.get_signature(entity);
        let component_type = match self.component_manager.get_component_type::<T>() {
            Some(component_type) => component_type,
            None => {
                error!(entity, "Component type not registered.");
                return None;
            }
        };
        signature.set(component_type, true);
        self.entity_manager.set_signature(entity, signature.clone());

        Some(self)
    }

    /// Removes a component from an entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity index to remove the component from.
    /// 
    /// # Returns
    /// 
    /// * `&mut Self` - The world.
    /// * `None` - If the component type is not registered.
    pub fn remove_component<T: 'static>(&mut self, entity: EntityIndex) -> Option<&mut Self> {
        // Remove the component for the entity
        let ans = self.component_manager.remove_component::<T>(entity);
        if ans.is_err() {
            return None;
        }

        // Remove the component from the entity's signature
        let mut signature = self.entity_manager.get_signature(entity);
        signature.set(self.component_manager.get_component_type::<T>().unwrap(), false);
        self.entity_manager.set_signature(entity, signature.clone());

        Some(self)
    }

    /// Gets the value of a component from an entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity index to get the component from.
    /// 
    /// # Returns
    /// 
    /// * `Some(T)` - The component value.
    /// * `None` - If the entity does not have the component.
    pub fn get_component<T: 'static>(&self, entity: EntityIndex) -> Option<&T> {
        self.component_manager.get_component::<T>(entity)
    }

    /// Sets the value of a component for an entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity index to set the component for.
    /// * `component` - The component to set.
    /// 
    /// # Returns
    /// 
    /// * `&mut Self` - The world.
    /// * `None` - If the component type is not registered.
    pub fn set_component<T: 'static + std::fmt::Debug>(&mut self, entity: EntityIndex, component: T) -> Option<&mut Self> {
        let ans = self.component_manager.set_component::<T>(entity, component);
        if ans.is_err() {
            return None;
        }
        return Some(self);
    }

    /// Gets the component type of a generic component.
    /// 
    /// # Returns
    /// 
    /// * `Some(ComponentIndex)` - The component type of the component.
    /// * `None` - If the component type is not registered.
    pub fn get_component_type<T: 'static>(&self) -> Option<ComponentIndex> {
        self.component_manager.get_component_type::<T>();
        None
    }
}
