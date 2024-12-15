use bevy::prelude::*;
use wde_render::{assets::{materials::*, meshes::CubeGizmoMesh, Mesh, ModelBoundingBox}, components::*};
use wde_wgpu::vertex::WVertex;

use super::marching_cubes_patterns::{MARCHING_CUBES_CORNER_INDEX_A_FROM_EDGE, MARCHING_CUBES_CORNER_INDEX_B_FROM_EDGE, MARCHING_CUBES_TRIANGLES};

pub struct MarchingCubesPlugin;
impl Plugin for MarchingCubesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, init);
    }
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
    let chunks_count = 10;
    let grid_scale = 1.0;
    let iso_level = 0.0;

    // Define the scalar field
    let center_tr = -0.5;
    let circle_radius = 2.0;
    let f = |p: Vec3| -> f32 {
        (p.x - center_tr) * (p.x - center_tr) + (p.y - center_tr) * (p.y - center_tr)
        + (p.z - center_tr) * (p.z - center_tr) - circle_radius * circle_radius
    };

    // Generate the gizmo cubes
    let cube = asset_server.add(CubeGizmoMesh::from("Marching cubes", 1.0));
    for i in 0..chunks_count {
        for j in 0..chunks_count {
            for k in 0..chunks_count {
                let x = (i as f32 - chunks_count as f32 / 2.0) * grid_scale;
                let y = (j as f32 - chunks_count as f32 / 2.0) * grid_scale;
                let z = (k as f32 - chunks_count as f32 / 2.0) * grid_scale;
                let position = Vec3::new(x, y, z);
                commands.spawn(GizmoBundle {
                    transform: Transform::from_translation(position).with_scale(Vec3::splat(grid_scale)),
                    mesh: cube.clone(),
                    material: gizmo_edges.clone()
                });
            }
        }
    }

    // Generate the marching cube meshes
    for i in 0..chunks_count {
        for j in 0..chunks_count {
            for k in 0..chunks_count {
                let x = (i as f32 - chunks_count as f32 / 2.0) * grid_scale;
                let y = (j as f32 - chunks_count as f32 / 2.0) * grid_scale;
                let z = (k as f32 - chunks_count as f32 / 2.0) * grid_scale;
                let position = Vec3::new(x, y, z);
                if let Some(mesh) = generate_mesh_chunk(iso_level, position, grid_scale, chunks_count, &f) {
                    commands.spawn(PbrBundle {
                        mesh: asset_server.add(mesh),
                        material: blue.clone(),
                        transform: Transform::IDENTITY
                    });
                }
            }
        }
    }
}


/**
 * Generate a mesh chunk at a given position given a grid size.
 * 
 * # Arguments
 * 
 * `iso_level` - The value of the isosurface.
 * `translation` - The translation of the mesh.
 * `scale` - The scale of the mesh.
 * `chunks_size` - The size of the chunk in each dimension.
 * `f` - The function to evaluate the scalar field at a given point.
 * 
 * # Returns
 * 
 * The generated mesh.
 */
fn generate_mesh_chunk(iso_level: f32, translation: Vec3, scale: f32, chunks_size: usize, f: &dyn Fn(Vec3) -> f32) -> Option<Mesh> {
    // Generate the grid points
    let mut points = Vec::new();
    for i in 0..chunks_size {
        for j in 0..chunks_size {
            for k in 0..chunks_size {
                let x = (i as f32 - chunks_size as f32 / 2.0 - 1.0) * scale + translation.x;
                let y = (j as f32 - chunks_size as f32 / 2.0) * scale + translation.y;
                let z = (k as f32 - chunks_size as f32 / 2.0) * scale + translation.z;
                let value = f(Vec3::new(x, y, z));
                points.push([x, y, z, value]);
            }
        }
    }
    
    // Generate the triangles
    let mut triangles = Vec::new();
    for i in 0..chunks_size-1 {
        for j in 0..chunks_size-1 {
            for k in 0..chunks_size-1 {
                // Generate the mesh
                let new_triangles = generate_mesh_cube(&points, iso_level, i, j, k, chunks_size);
                if new_triangles.is_none() {
                    continue;
                }
                for triangle in new_triangles.unwrap().iter() {
                    triangles.push(*triangle);
                }
            }
        }
    }

    // Generate the mesh
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut bounding_box = ModelBoundingBox::default();
    for triangle in triangles.iter() {
        // Add the vertices
        let i0 = vertices.len() as u32;
        vertices.push(WVertex {
            position: [triangle[0][0], triangle[0][1], triangle[0][2]],
            normal: [0.0, 0.0, 0.0],
            uv: [0.0, 0.0]
        });
        let i1 = vertices.len() as u32;
        vertices.push(WVertex {
            position: [triangle[1][0], triangle[1][1], triangle[1][2]],
            normal: [0.0, 0.0, 0.0],
            uv: [0.0, 0.0]
        });
        let i2 = vertices.len() as u32;
        vertices.push(WVertex {
            position: [triangle[2][0], triangle[2][1], triangle[2][2]],
            normal: [0.0, 0.0, 0.0],
            uv: [0.0, 0.0]
        });
        indices.push(i0);
        indices.push(i1);
        indices.push(i2);

        // Update the bounding box
        (0..3).for_each(|i| {
            bounding_box.min[i] = bounding_box.min[i].min(triangle[i][0]);
            bounding_box.min[i] = bounding_box.min[i].min(triangle[i][1]);
            bounding_box.min[i] = bounding_box.min[i].min(triangle[i][2]);
            bounding_box.max[i] = bounding_box.max[i].max(triangle[i][0]);
            bounding_box.max[i] = bounding_box.max[i].max(triangle[i][1]);
            bounding_box.max[i] = bounding_box.max[i].max(triangle[i][2]);
        });

        // Compute the normal
        let v0 = Vec3::new(triangle[0][0], triangle[0][1], triangle[0][2]);
        let v1 = Vec3::new(triangle[1][0], triangle[1][1], triangle[1][2]);
        let v2 = Vec3::new(triangle[2][0], triangle[2][1], triangle[2][2]);
        let normal = (v1 - v0).cross(v2 - v0).normalize();
        vertices[i0 as usize].normal = [normal.x, normal.y, normal.z];
        vertices[i1 as usize].normal = [normal.x, normal.y, normal.z];
        vertices[i2 as usize].normal = [normal.x, normal.y, normal.z];
    }

    Some(Mesh {
        label: "marching-cube-chunk".to_string(),
        vertices,
        indices,
        bounding_box
    })
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
 * 
 * # Returns
 * 
 * The generated mesh.
 */
fn generate_mesh_cube(points: &[[f32; 4]], iso_level: f32, i: usize, j: usize, k: usize, chunks_size: usize) -> Option<Vec<[[f32; 3]; 3]>> {
    // Compute the values of the vertices of the cube
    let cube_corners = [
        points[index_from_coord(i, j, k, chunks_size)],
        points[index_from_coord(i + 1, j, k, chunks_size)],
        points[index_from_coord(i + 1, j, k + 1, chunks_size)],
        points[index_from_coord(i, j, k + 1, chunks_size)],
        points[index_from_coord(i, j + 1, k, chunks_size)],
        points[index_from_coord(i + 1, j + 1, k, chunks_size)],
        points[index_from_coord(i + 1, j + 1, k + 1, chunks_size)],
        points[index_from_coord(i, j + 1, k + 1, chunks_size)]
    ];

    // Compute cube index based on the values of the vertices
    let mut cube_index = 0;
    if cube_corners[0][3] < iso_level { cube_index |= 1; }
    if cube_corners[1][3] < iso_level { cube_index |= 2; }
    if cube_corners[2][3] < iso_level { cube_index |= 4; }
    if cube_corners[3][3] < iso_level { cube_index |= 8; }
    if cube_corners[4][3] < iso_level { cube_index |= 16; }
    if cube_corners[5][3] < iso_level { cube_index |= 32; }
    if cube_corners[6][3] < iso_level { cube_index |= 64; }
    if cube_corners[7][3] < iso_level { cube_index |= 128; }

    // Discard the case where the cube is entirely inside or outside the surface
    if cube_index == 0 || cube_index == 255 {
        return None;
    }

    // Polygonise the cube
    let mut i = 0;
    let mut triangles = Vec::new();
    loop {
        if MARCHING_CUBES_TRIANGLES[cube_index][i] == -1 {
            break;
        }

        // Get indices of corner points A and B for each of the three edges
        // of the cube that need to be joined to form the triangle.
        let a0 = MARCHING_CUBES_CORNER_INDEX_A_FROM_EDGE[MARCHING_CUBES_TRIANGLES[cube_index][i] as usize];
        let b0 = MARCHING_CUBES_CORNER_INDEX_B_FROM_EDGE[MARCHING_CUBES_TRIANGLES[cube_index][i] as usize];

        let a1 = MARCHING_CUBES_CORNER_INDEX_A_FROM_EDGE[MARCHING_CUBES_TRIANGLES[cube_index][i+1] as usize];
        let b1 = MARCHING_CUBES_CORNER_INDEX_B_FROM_EDGE[MARCHING_CUBES_TRIANGLES[cube_index][i+1] as usize];

        let a2 = MARCHING_CUBES_CORNER_INDEX_A_FROM_EDGE[MARCHING_CUBES_TRIANGLES[cube_index][i+2] as usize];
        let b2 = MARCHING_CUBES_CORNER_INDEX_B_FROM_EDGE[MARCHING_CUBES_TRIANGLES[cube_index][i+2] as usize];

        // Interpolate the vertices
        triangles.push([
            vertex_interpolate(iso_level, cube_corners[a0 as usize], cube_corners[b0 as usize]),
            vertex_interpolate(iso_level, cube_corners[a2 as usize], cube_corners[b2 as usize]),
            vertex_interpolate(iso_level, cube_corners[a1 as usize], cube_corners[b1 as usize])
        ]);
        i += 3;
    }
    Some(triangles)
}

/**
 * Get the index of a point in the grid from its coordinates.
 */
fn index_from_coord(x: usize, y: usize, z: usize, chunks_size: usize) -> usize {
    z * chunks_size * chunks_size + y * chunks_size + x
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

