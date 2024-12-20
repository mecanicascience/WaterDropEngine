use bevy::prelude::*;
use wde_render::{assets::{materials::*, meshes::CubeGizmoMesh, Mesh, ModelBoundingBox}, components::*, utils::Color};
use wde_wgpu::vertex::WVertex;

use super::{mc_compute_core::MarchingCubesPluginCompute, old_mc_patterns::*};
use noise::{NoiseFn, Perlin};

pub struct MarchingCubesPlugin;
impl Plugin for MarchingCubesPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, init);

        app.add_plugins(MarchingCubesPluginCompute);
    }
}


fn generate_perlin_noise(x: f32, y: f32, z: f32, scale: f32, seed: u32) -> f32 {
    let perlin = Perlin::new(seed);
    perlin.get([x as f64 * scale as f64, y as f64 * scale as f64, z as f64 * scale as f64]) as f32
}

/**
 * System to initialize the game.
 */
fn init(
    mut commands: Commands, asset_server: Res<AssetServer>,
    mut pbr_materials: ResMut<Assets<PbrMaterial>>,
    mut gizmo_materials: ResMut<Assets<GizmoMaterial>>
) {
    // Main camera
    commands.spawn(
        (Camera {
            transform: Transform::from_xyz(5.0, 5.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            view: CameraView::default()
        },
        CameraController::default(),
        ActiveCamera
    ));
    
    // Light
    commands.spawn(DirectionalLight {
        direction: Vec3::new(0.1, -0.8, 0.2),
        ambient: Color::from_srgba(0.3, 0.3, 0.3, 1.0),
        ..Default::default()
    });
    commands.spawn(DirectionalLight {
        direction: Vec3::new(-0.3, 0.8, -0.5),
        ambient: Color::from_srgba(0.1, 0.1, 0.1, 1.0),
        ..Default::default()
    });

    // Load the materials
    let gizmo_edges = gizmo_materials.add(GizmoMaterial {
        label: "gizmo-edges".to_string(),
        color: [0.0, 1.0, 0.0, 1.0]
    });
    let blue = pbr_materials.add(PbrMaterial {
        label: "blue".to_string(),
        albedo: (0.0, 0.0, 1.0, 1.0),
        specular: 0.5,
        ..Default::default()
    });


    // Create a list of positions in a grid around the origin
    let chunks_count = 2;
    let chunk_length = 2.0;
    let chunk_sub_count = 10;

    // Define the scalar field
    let iso_level = 0.0;
    let f = |p: Vec3| -> f32 {
        generate_perlin_noise(p.x, p.y, p.z, 1.0, 0)
    };

    // Generate the marching cube meshes
    let mut meshes = Vec::new();
    for i in 0..chunks_count {
        for j in 0..chunks_count {
            for k in 0..chunks_count {
                // Compute the position of the chunk
                let tot_scale = chunk_length * (chunks_count as f32);
                let translation = Vec3::new(
                    -tot_scale / 2.0 + i as f32 * chunk_length,
                    -tot_scale / 2.0 + j as f32 * chunk_length,
                    -tot_scale / 2.0 + k as f32 * chunk_length
                );

                // Generate the mesh
                let mesh = generate_mesh_chunk(iso_level, translation, chunk_length, chunk_sub_count, &f);
                info!("Generated mesh at {:?} with {} vertices", translation, mesh.vertices.len());
                meshes.push(mesh);
            }
        }
    }

    // Draw a gizmo corresponding to each bounding box
    for mesh in &meshes {
        let min = mesh.bounding_box.min;
        let max = mesh.bounding_box.max;
        let center = (min + max) / 2.0;
        let size = max - min;
        let cube = asset_server.add(CubeGizmoMesh::from("Marching cubes", size.x));
        commands.spawn(GizmoBundle {
            transform: Transform::from_translation(center),
            mesh: cube,
            material: gizmo_edges.clone()
        });
    }

    for mesh in meshes {
        commands.spawn(PbrBundle {
            transform: Transform::default(),
            mesh: asset_server.add(mesh),
            material: blue.clone()
        });
    }
}


/**
 * Generate a mesh chunk at a given position given a grid size.
 * 
 * # Arguments
 * 
 * `iso_level` - The value of the isosurface.
 * `translation` - The translation of the mesh.
 * `scale` - The length of a grid cell.
 * `chunk_sub_count` - The number of grid cells in each dimension.
 * `f` - The function to evaluate the scalar field at a given point.
 * 
 * # Returns
 * 
 * The generated mesh.
 */
fn generate_mesh_chunk(iso_level: f32, translation: Vec3, chunk_length: f32, chunk_sub_count: usize, f: &dyn Fn(Vec3) -> f32) -> Mesh {
    // Generate the grid points
    let mut points = Vec::with_capacity(chunk_sub_count * chunk_sub_count * chunk_sub_count);
    for i in 0..chunk_sub_count {
        for j in 0..chunk_sub_count {
            for k in 0..chunk_sub_count {
                let x = translation.x - chunk_length / 2.0 + i as f32 * chunk_length / (chunk_sub_count as f32 - 1.0);
                let y = translation.y - chunk_length / 2.0 + j as f32 * chunk_length / (chunk_sub_count as f32 - 1.0);
                let z = translation.z - chunk_length / 2.0 + k as f32 * chunk_length / (chunk_sub_count as f32 - 1.0);
                points.push([x, y, z, f(Vec3::new(x, y, z))]);
            }
        }
    }
    
    // Generate the vertices
    let mut vertices: Vec<WVertex> = vec![WVertex::default(); 3 * 5 * chunk_sub_count * chunk_sub_count * chunk_sub_count];
    let mut count = 0;
    for i in 0..chunk_sub_count-1 {
        for j in 0..chunk_sub_count-1 {
            for k in 0..chunk_sub_count-1 {
                generate_mesh_cube(&points, iso_level, i, j, k, chunk_sub_count, &mut vertices, &mut count);
            }
        }
    }

    // Compute the bounding box (BAD : @TODO to change)
    let mut bounding_box = ModelBoundingBox::default();
    for vertex in &vertices[..count] {
        bounding_box.min.x = bounding_box.min.x.min(vertex.position[0]);
        bounding_box.min.y = bounding_box.min.y.min(vertex.position[1]);
        bounding_box.min.z = bounding_box.min.z.min(vertex.position[2]);
        bounding_box.max.x = bounding_box.max.x.max(vertex.position[0]);
        bounding_box.max.y = bounding_box.max.y.max(vertex.position[1]);
        bounding_box.max.z = bounding_box.max.z.max(vertex.position[2]);
    }

    // Create the mesh
    Mesh {
        label: format!("marching-cubes-{:?}", translation),
        vertices: vertices[..count].to_vec(),
        indices:  (0..count as u32).collect(),
        bounding_box
    }
}

/**
 * Generate a mesh at a given cube position.
 * 
 * # Arguments
 * 
 * `points` - The list of points in the grid [x, y, z, value].
 * `iso_level` - The value of the isosurface.
 * `i` - The x coordinate of the grid cell.
 * `j` - The y coordinate of the grid cell.
 * `k` - The z coordinate of the grid cell.
 * `chunks_size` - The size of the chunk in each dimension.
 * `vertices` - The list of vertices to fill.
 * `count` - The number of vertices in the list.
 */
#[allow(clippy::too_many_arguments)]
fn generate_mesh_cube(points: &[[f32; 4]], iso_level: f32, i: usize, j: usize, k: usize, chunk_sub_count: usize, vertices: &mut [WVertex], count: &mut usize) {
    // Compute the values of the vertices of the cube
    let mut cube_corners = [[0.0; 4]; 8];
    MARCHING_CUBES_CORNER_INDICES.iter().enumerate().for_each(|(l, [di, dj, dk])| {
        cube_corners[l] = points[index_from_coord(i + di, j + dj, k + dk, chunk_sub_count)];
    });

    // Compute cube index based on the values of the vertices
    let mut cube_index = 0;
    (0..8).for_each(|l| {
        if cube_corners[l][3] < iso_level {
            cube_index |= 1 << l;
        }
    });

    // Polygonise the cube
    for i in (0..16).step_by(3) {
        if MARCHING_CUBES_TRIANGLES[cube_index][i] == -1 {
            break;
        }

        // Get the triangle vertices
        let mut position = [[0.0; 3]; 3];
        for j in 0..3 {
            let a = MARCHING_CUBES_CORNER_INDEX_A_FROM_EDGE[MARCHING_CUBES_TRIANGLES[cube_index][i+(2-j)] as usize];
            let b = MARCHING_CUBES_CORNER_INDEX_B_FROM_EDGE[MARCHING_CUBES_TRIANGLES[cube_index][i+(2-j)] as usize];
            let interp = vertex_interpolate(iso_level, cube_corners[a as usize], cube_corners[b as usize]);
            position[j] = interp;
        }

        // Compute the normal
        let v0 = Vec3::new(position[0][0], position[0][1], position[0][2]);
        let v1 = Vec3::new(position[1][0], position[1][1], position[1][2]);
        let v2 = Vec3::new(position[2][0], position[2][1], position[2][2]);
        let normal = (v1 - v0).cross(v2 - v0).normalize();

        // Add the triangle
        for j in 0..3 {
            vertices[*count + j] = WVertex {
                position: [position[j][0], position[j][1], position[j][2]],
                normal: [normal.x, normal.y, normal.z],
                uv: [0.0, 0.0]
            };
        }
        *count += 3;
    }
}

/**
 * Get the index of a point in the grid from its coordinates.
 * 
 * # Arguments
 * 
 * * `x` - The x coordinate.
 * * `y` - The y coordinate.
 * * `z` - The z coordinate.
 * * `chunk_sub_count` - The number of grid cells in each dimension.
 */
fn index_from_coord(x: usize, y: usize, z: usize, chunk_sub_count: usize) -> usize {
    z * chunk_sub_count * chunk_sub_count + y * chunk_sub_count + x
}

/**
 * Linearly interpolate the position where an isosurface cuts
 * an edge between two vertices, each with their own scalar value.
 * 
 * # Arguments
 * 
 * * `iso_level` - The iso_level.
 * * `p1` - The first vertex and its scalar value.
 * * `p2` - The second vertex and its scalar value.
 * 
 * # Returns
 * 
 * The interpolated position.
 */
fn vertex_interpolate(iso_level: f32, p1: [f32; 4], p2: [f32; 4]) -> [f32; 3] {
    let t = (iso_level - p1[3]) / (p2[3] - p1[3]);
    [
        p1[0] + t * (p2[0] - p1[0]),
        p1[1] + t * (p2[1] - p1[1]),
        p1[2] + t * (p2[2] - p1[2])
    ]
}

