use bevy::prelude::*;
use wde_render::assets::{materials::{GizmoMaterial, GizmoMaterialAsset}, meshes::CubeGizmoMesh, Mesh};

use crate::terrain::mc_chunk::{MCSpawnEvent, MCChunkDescription, MC_MAX_SUB_COUNT};

pub struct MarchingCubesSpawner;
impl MarchingCubesSpawner {
    /**
     * Manage the chunks creation / deletion.
     * This create a task for each chunk to generate the mesh.
     * This will then generate the points, vertices and indices buffers, and add the chunk to the loading chunks.
     */
    pub fn manage_chunks(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut gizmo_materials: ResMut<Assets<GizmoMaterialAsset>>,
        mut chunks_spawn_events: EventWriter<MCSpawnEvent>,
    ) {
        // Chunks grid
        let chunks_count = 5;
        let chunk_length = [500.0, 200.0, 500.0];
        let chunk_sub_count = MC_MAX_SUB_COUNT;
        let iso_level = 0.0;

        // Spawn the chunks
        for i in 0..chunks_count {
            for k in 0..chunks_count {
                // Compute the position of the chunk
                let tot_scale = [chunk_length[0] * chunks_count as f32, chunk_length[1], chunk_length[2] * chunks_count as f32];
                let translation = Vec3::new(
                    -tot_scale[0] / 2.0 + i as f32 * chunk_length[0],
                    0.0,
                    -tot_scale[2] / 2.0 + k as f32 * chunk_length[2]
                );

                // Spawn the chunk
                chunks_spawn_events.send(MCSpawnEvent(MCChunkDescription {
                    index: (i, 0, k),
                    translation,
                    chunk_length: chunk_length.into(),
                    chunk_sub_count: chunk_sub_count.into(),
                    iso_level
                })); 
            }
        }

        // Draw a gizmo corresponding to each bounding box
        let gizmo_edges = gizmo_materials.add(GizmoMaterialAsset {
            label: "gizmo-edges".to_string(),
            color: [0.0, 1.0, 0.0, 1.0]
        });
        let cube = asset_server.add(CubeGizmoMesh::from("Marching cubes", chunk_length.into()));
        for i in 0..chunks_count {
            for k in 0..chunks_count {
                let translation = Vec3::new(
                    -chunk_length[0] * (chunks_count as f32) / 2.0 + i as f32 * chunk_length[0],
                    0.0,
                    -chunk_length[2] * (chunks_count as f32) / 2.0 + k as f32 * chunk_length[2]
                );
                commands.spawn((
                    Transform::from_translation(translation),
                    Mesh(cube.clone()),
                    GizmoMaterial(gizmo_edges.clone())
                ));
            }
        }
    }
}