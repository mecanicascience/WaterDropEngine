use std::collections::VecDeque;

use wde_logger::{info, error};

use crate::{ComponentSignature, EntityIndex, MAX_ENTITIES, world};


/// Manages the creation and destruction of a set of entities.
/// This stores the list of all entities and their components.
/// 
/// # Example
/// 
/// ```
/// // Create a new entity manager
/// let mut entity_manager = EntityManager::new();
/// 
/// // Create a new entity
/// let entity = entity_manager.create_entity();
/// 
/// // Add the component 1 to the entity
/// let mut signature = world::empty_signature();
/// signature.set(1, true);
/// entity_manager.set_signature(entity, signature);
/// 
/// // Get the signature of the entity
/// let signature = entity_manager.get_signature(entity);
/// ```
pub struct EntityManager {
    /// List of components for each entity.
    pub components_list: Vec<ComponentSignature>,

    /// List of used entities.
    pub living_entities: Vec<EntityIndex>,
    /// Number of entities living in the manager.
    pub living_entity_count: u32,
    /// List of unused entities.
    pub dead_entities: VecDeque<EntityIndex>,
}

impl EntityManager {
    pub fn new() -> Self {
        info!("Creating a new entity manager.");

        Self {
            components_list: Vec::with_capacity(MAX_ENTITIES as usize),
            living_entities: Vec::with_capacity(MAX_ENTITIES as usize),
            living_entity_count: 0,
            dead_entities: (0..=MAX_ENTITIES-1).collect()
        }
    }

    /// Creates a new entity and returns its id.
    /// Note that this function does not add any components to the entity and does not set its signature.
    /// If no more entities are available, this function will return `None`.
    pub fn create_entity(&mut self) -> Option<EntityIndex> {
        // Get the next unused entity
        let entity = match self.dead_entities.pop_front() {
            Some(entity) => entity,
            None => {
                error!("No more entities available");
                return None;
            }
        };
        self.living_entity_count += 1;
        self.living_entities.push(entity);

        // Add a new empty signature for the new entity, by default it has no components
        self.components_list.push(world::empty_signature());

        Some(entity)
    }

    /// Destroys an entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity to destroy.
    pub fn destroy_entity(&mut self, entity: EntityIndex) {
        // Clear the signature of the destroyed entity
        self.components_list[entity].clear();
        
        // Add the destroyed entity id to the unused entities queue
        self.dead_entities.push_back(entity);
        self.living_entities.retain(|&e| e != entity);
        self.living_entity_count -= 1;
    }


    /// Sets the components signature of an entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity to set the signature of.
    /// * `signature` - The signature to set.
    pub fn set_signature(&mut self, entity: EntityIndex, signature: ComponentSignature) {
        self.components_list[entity] = signature;
    }

    /// Gets the signature of an entity.
    /// 
    /// # Arguments
    /// 
    /// * `entity` - The entity to get the signature of.
    pub fn get_signature(&self, entity: EntityIndex) -> ComponentSignature {
        self.components_list[entity].clone()
    }

    /// Gets the list of all living entities.
    pub fn get_all_entities(&self) -> &Vec<EntityIndex> {
        &self.living_entities
    }
}
