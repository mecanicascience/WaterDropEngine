use std::collections::HashMap;

use bevy::prelude::*;

use crate::assets::{materials::TerrainChunkMaterial, meshes::PlaneMesh, Mesh};

use super::ActiveCamera;

/** Id of a chunk */
pub type ChunkId = (i32, i32);

/** Handles the generation of chunks around this position. */
#[derive(Component)]
pub struct ChunkSpawner {
    /** Chunk ID of the spawner. */
    pub main_chunk_id: ChunkId,
    /** Number of chunks to spawn around the spawner in a circle. */
    pub chunk_radius: i32,
    /** Size of each chunk. */
    pub chunk_size: usize,

    /** List of chunks that have been spawned. */
    pub spawned_chunks: HashMap<ChunkId, Entity>,
}
impl Default for ChunkSpawner {
    fn default() -> Self {
        ChunkSpawner {
            main_chunk_id: (0, 0),
            chunk_radius: 4,
            chunk_size: 64,

            spawned_chunks: HashMap::new(),
        }
    }
}

pub struct ChunkSpawnerPlugin;
impl Plugin for ChunkSpawnerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (move_spawner, spawn_chunks).chain());
    }
}

/** Moves the spawner to the player position. */
fn move_spawner(
    mut chunk_spawner_query: Query<&mut ChunkSpawner>,
    player_query: Query<&Transform, With<ActiveCamera>>,
) {
    let player_transform = match player_query.get_single() {
        Ok(value) => value,
        Err(_) => return,
    };

    let mut chunk_spawner = match chunk_spawner_query.get_single_mut() {
        Ok(value) => value,
        Err(_) => return,
    };
    chunk_spawner.main_chunk_id = get_chunk_id(player_transform, chunk_spawner.chunk_size);
}

/** Spawns chunks around the spawner. */
fn spawn_chunks(
    mut commands: Commands,
    mut chunk_spawner_query: Query<&mut ChunkSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainChunkMaterial>>,
) {
    let mut chunk_spawner = match chunk_spawner_query.get_single_mut() {
        Ok(value) => value,
        Err(_) => return,
    };
    let chunk_id = chunk_spawner.main_chunk_id;
    let chunk_radius = chunk_spawner.chunk_radius;

    // List of all currently spawned chunks
    let mut spawned_chunks = chunk_spawner.spawned_chunks.clone();

    // Spawn chunks around the spawner
    for x in -chunk_radius..=chunk_radius {
        for z in -chunk_radius..=chunk_radius {
            // Check if the chunk is in the radius
            if x * x + z * z > chunk_radius * chunk_radius {
                continue;
            }

            // Check if the chunk is already spawned
            let chunk_id = (chunk_id.0 + x, chunk_id.1 + z);
            if chunk_spawner.spawned_chunks.contains_key(&chunk_id) {
                spawned_chunks.remove(&chunk_id);
                continue;
            }
            debug!("Spawning terrain chunk {:?}.", chunk_id);

            // Spawn the chunk
            let mesh = meshes.add(PlaneMesh::from(format!("chunk_{}_{}", chunk_id.0, chunk_id.1).as_str(), [chunk_spawner.chunk_size as f32, chunk_spawner.chunk_size as f32]));
            let material = materials.add(TerrainChunkMaterial {
                chunk_id,
                albedo: None,
            });
            let chunk = Chunk {
                chunk_id,
                mesh,
                material,
            };
            let entity = commands.spawn((chunk, Transform::from_xyz(
                chunk_id.0 as f32 * chunk_spawner.chunk_size as f32,
                0.0,
                chunk_id.1 as f32 * chunk_spawner.chunk_size as f32,
            ))).id();
            chunk_spawner.spawned_chunks.insert(chunk_id, entity);
        }
    }

    // Despawn chunks that are no longer in the radius
    for (_, entity) in spawned_chunks.iter() {
        debug!("Despawning terrain chunk {:?}.", entity);
        commands.entity(*entity).despawn();
    }
}



/** Describes a chunk property. */
#[derive(Component)]
pub struct Chunk {
    /** Chunk ID. */
    pub chunk_id: ChunkId,
    /** Mesh of the chunk. */
    pub mesh: Handle<Mesh>,
    /** Material of the chunk. */
    pub material: Handle<TerrainChunkMaterial>,
}

/**
 * Returns the chunk ID of the given transform position.
 */
pub fn get_chunk_id(transform: &Transform, chunk_size: usize) -> ChunkId {
    let x = transform.translation.x as i32 / chunk_size as i32;
    let z = transform.translation.z as i32 / chunk_size as i32;
    (x, z)
}
