use bevy::{prelude::*, utils::HashSet};

#[derive(Default, Debug, Resource)]
/// Manages the chunks of a given world instance
pub struct ChunkManager {
    /// List of currently loaded chunks
    pub chunks: HashSet<IVec2>
}


/// Number of tiles in a chunk in one direction
const CHUNK_SIZE: usize = 16;

#[derive(Default, Component)]
/// Describes a chunk of the world that stores a set of tiles
pub struct Chunk {
    /// Chunk coordinates (top left tile of (0, 0) is the center of the world)
    pub index: IVec2,
    /// List of tiles in the chunk
    pub tiles: [[Option<Entity>; CHUNK_SIZE]; CHUNK_SIZE]
}


#[derive(Default, Component)]
/// Describes a tile in a chunk
pub struct Tile {
    /// Position of the tile in the chunk
    pub position: Vec2
}


pub fn spawn_chunk_around_pos(commands: &mut Commands, mut chunk_manager: ResMut<ChunkManager>) {
    let pos = IVec2::new(5, -2);

    if !chunk_manager.chunks.contains(&pos) {
        // Spawn a new chunk
        commands.spawn(Chunk {
            index: pos,
            ..Default::default()
        });

        // Register the chunk in the chunk manager
        chunk_manager.chunks.insert(pos);
    }
}



#[derive(Default, Component)]
pub struct DummyComponent;

fn get_all_dummy_tiles(commands: &mut Commands, dummy_times: Query<(Entity, &Tile, &DummyComponent)>) {
    // For each dummy tile
    for _ in dummy_times.iter() {
    }
}
