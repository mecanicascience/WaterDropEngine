use bevy::math::Vec3;
use wde_wgpu::vertex::WVertex;

use crate::assets::{Mesh, ModelBoundingBox};

pub struct CubeGizmoMesh;
impl CubeGizmoMesh {
    /// Create a new cube gizmo mesh.
    /// The cube goes from (-length/2) to (length/2) in all directions.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label for the mesh.
    /// * `length` - The length of the cube's sides.
    /// 
    /// # Returns
    /// 
    /// The cube gizmo mesh.
    pub fn from(label: &str, length: f32) -> Mesh {
        let half_length = length / 2.0;

        // Create vertices
        let positions = [
            // Front face
            -half_length, -half_length,  half_length,
             half_length, -half_length,  half_length,
             half_length,  half_length,  half_length,
            -half_length,  half_length,  half_length,
            // Back face
            -half_length, -half_length, -half_length,
             half_length, -half_length, -half_length,
             half_length,  half_length, -half_length,
            -half_length,  half_length, -half_length,
        ];

        let mut vertices = Vec::new();
        for vtx in 0..positions.len() / 3 {
            let x = positions[3 * vtx];
            let y = positions[3 * vtx + 1];
            let z = positions[3 * vtx + 2];

            // Vertex
            vertices.push(WVertex {
                position: [x, y, z],
                normal: [0.0, 0.0, 0.0], // Normals are not used for gizmo
                uv: [0.0, 0.0], // UVs are not used for gizmo
            });
        }

        // Create indices for line list topology
        let indices = vec![
            // Front face
            0, 1, 1, 2, 2, 3, 3, 0,
            // Back face
            4, 5, 5, 6, 6, 7, 7, 4,
            // Connections
            0, 4, 1, 5, 2, 6, 3, 7,
        ];

        // Create bounding box
        let bounding_box = ModelBoundingBox {
            min: Vec3::new(-half_length, -half_length, -half_length),
            max: Vec3::new( half_length,  half_length,  half_length),
        };

        Mesh {
            label: label.to_string(),
            vertices,
            indices,
            bounding_box,
        }
    }
}
