use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use noise::NoiseFn;
use render::mc_compute_core::{MarchingCubesChunkDescription, MarchingCubesComputeTask};
use wde_render::{assets::{materials::{GizmoMaterial, GizmoMaterialAsset}, meshes::CubeGizmoMesh, Mesh}, core::DeviceLimits};

mod render;

pub struct TerrainPlugin;
impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // Add the render plugin
        app.add_plugins(render::TerrainFeaturesPlugin);

        // Add the generation of the chunks
        app.add_systems(Update, manage_chunks.run_if(run_once));
    }
}

/**
 * Manage the chunks creation / deletion.
 * This create a task for each chunk to generate the mesh.
 * This will then generate the points, vertices and indices buffers, and add the chunk to the loading chunks.
 */
pub fn manage_chunks(
    mut commands: Commands,
    gpu_limits: Res<DeviceLimits>,
    asset_server: Res<AssetServer>,
    mut gizmo_materials: ResMut<Assets<GizmoMaterialAsset>>
) {
    // Chunks grid
    let chunks_count = 5;
    let chunk_length = [500.0, 200.0, 500.0];
    let chunk_sub_count = [150, 20, 150];
    let iso_level = 0.0;

    // Terrain function
    fn generate_perlin_noise(x: f32, y: f32, z: f32) -> f32 {
        let mut amplitude = 150.0;
        let mut frequency = 0.005;
        let ground_percent = 0.1;

        let octaves = 8;
        let persistence = 0.5;
        let lacunarity = 2.0;

        let seed = 0;
        let simplex_noise = noise::OpenSimplex::new(seed);

        // Perlin noise parameters
        let chunk_length = [500.0, 100.0, 500.0];
        let mut height = 0.0;
        for _ in 0..octaves {
            height += amplitude * simplex_noise.get([x as f64 * frequency, y as f64 * frequency, z as f64 * frequency]);
            amplitude *= persistence;
            frequency *= lacunarity;
        }
        let ground = y + ground_percent * chunk_length[1];
        ground + height as f32

        // // Perlin noise parameters
        // let terrain_scale = 1.0 / 500.0;
        // let terrain_seed = 0;
        // // Generate the perlin noise
        // let perlin = Perlin::new(terrain_seed);
        // perlin.get([x as f64 * terrain_scale, y as f64 * terrain_scale, z as f64 * terrain_scale]) as f32

        // // Sphere
        // x * x + y * y + z * z - 3.0
    }

    // Generate the chunks
    let thread_pool = AsyncComputeTaskPool::get();
    let max_buffer_size = gpu_limits.0.max_storage_buffer_binding_size as usize;
    for i in 0..chunks_count {
        for k in 0..chunks_count {
            // Spawn the task
            let task_entity = commands.spawn_empty().id();
            let task = thread_pool.spawn(async move {
                // Compute the position of the chunk
                let tot_scale = [chunk_length[0] * chunks_count as f32, chunk_length[1], chunk_length[2] * chunks_count as f32];
                let translation = Vec3::new(
                    -tot_scale[0] / 2.0 + i as f32 * chunk_length[0],
                    0.0,
                    -tot_scale[2] / 2.0 + k as f32 * chunk_length[2]
                );

                // Generate the mesh
                let desc = MarchingCubesChunkDescription {
                    index: (i, 0, k),
                    translation,
                    chunk_length: chunk_length.into(),
                    chunk_sub_count: chunk_sub_count.into(),
                    f: |pos| generate_perlin_noise(pos.x, pos.y, pos.z),
                    iso_level
                };

                // Generate the mesh
                MarchingCubesComputeTask::generate_new_chunk(task_entity, desc, max_buffer_size)
            });
            debug!("Task spawned for generating chunk {:?}.", (i, 0, k));

            // Spawn new entity and add our new task as a component
            commands.entity(task_entity).insert(MarchingCubesComputeTask(task));
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
