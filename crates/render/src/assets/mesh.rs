use std::{fs::File, io::BufReader};

use bevy::{asset::{io::Reader, AssetLoader, LoadContext}, ecs::system::lifetimeless::SRes, prelude::*};
use thiserror::Error;
use serde::{Deserialize, Serialize};
use tobj::LoadError;
use wde_wgpu::{buffer::{BufferUsage, WBuffer}, instance::WRenderInstance, vertex::WVertex};

use super::render_assets::{PrepareAssetError, RenderAsset};

/// The bounding box of the model.
#[derive(Clone, Debug)]
pub struct ModelBoundingBox {
    /// The minimum point of the bounding box.
    pub min: Vec3,
    /// The maximum point of the bounding box.
    pub max: Vec3,
}
impl Default for ModelBoundingBox {
    fn default() -> Self {
        Self {
            min: Vec3::new(f32::MAX, f32::MAX, f32::MAX),
            max: Vec3::new(f32::MIN, f32::MIN, f32::MIN),
        }
    }
}

#[derive(Component, Default)]
pub struct Mesh(pub Handle<MeshAsset>);
#[derive(Asset, TypePath, Clone)]
pub struct MeshAsset {
    /// The label of the texture.
    pub label: String,
    /// The list of vertices
    pub vertices: Vec<WVertex>,
    /// The list of indices
    pub indices: Vec<u32>,
    /// The bounding box of the model
    pub bounding_box: ModelBoundingBox,
}

#[derive(Default)]
pub struct MeshLoader;

#[derive(Serialize, Deserialize)]
pub struct MeshLoaderSettings {
    /// The label of the mesh.
    pub label: String,
}

impl Default for MeshLoaderSettings {
    fn default() -> Self {
        Self { label: "".to_string() }
    }
}

#[derive(Debug, Error)]
pub enum MeshLoaderError {
    #[error("Could not load mesh: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for MeshLoader {
    type Asset = MeshAsset;
    type Settings = MeshLoaderSettings;
    type Error = MeshLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("Loading mesh on the CPU from {}.", load_context.asset_path());

        // Update the label from the path
        let label = if settings.label.is_empty() {
            load_context.asset_path().to_string()
        } else {
            settings.label.clone()
        };

        // Read the texture data
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        // Open file
        #[allow(clippy::blocks_in_conditions)]
        let load_res = match tobj::load_obj_buf(
            &mut BufReader::new(bytes.as_slice()),
            &tobj::LoadOptions {
                single_index: true,
                ..Default::default()
            },
            |p| {
                let f = match File::open(p.file_name().unwrap().to_str().unwrap()) {
                    Ok(f) => f,
                    Err(_) => return Err(LoadError::OpenFileFailed)
                };
                tobj::load_mtl_buf(&mut BufReader::new(f))
            }
        ) {
            Ok(res) => res,
            Err(e) => return Err(MeshLoaderError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))),
        };
        let models = load_res.0;

        // Bounding box of the model
        let mut bounding_box = ModelBoundingBox {
            min: Vec3::new(f32::MAX, f32::MAX, f32::MAX),
            max: Vec3::new(f32::MIN, f32::MIN, f32::MIN),
        };

        // Load models
        let mut vertices: Vec<WVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for m in models.iter() {
            let mesh = &m.mesh;
            if mesh.positions.len() % 3 != 0 {
                return Err(MeshLoaderError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Mesh positions are not divisible by 3.")));
            }

            // Allocate sizes
            vertices.reserve(mesh.positions.len() / 3);

            // Create vertices
            for vtx in 0..mesh.positions.len() / 3 {
                let x = mesh.positions[3 * vtx];
                let y = mesh.positions[3 * vtx + 1];
                let z = mesh.positions[3 * vtx + 2];

                // Normals
                let mut nx = 0.0;
                let mut ny = 0.0;
                let mut nz = 0.0;
                if mesh.normals.len() >= 3 * vtx + 2 {
                    nx = mesh.normals[3 * vtx];
                    ny = mesh.normals[3 * vtx + 1];
                    nz = mesh.normals[3 * vtx + 2];
                }

                // UVs
                let mut u = 0.0;
                let mut v = 0.0;
                if mesh.texcoords.len() > 2 * vtx {
                    u = mesh.texcoords[2 * vtx];
                    v = mesh.texcoords[2 * vtx + 1];
                }

                // Vertex
                vertices.push(WVertex {
                    position: [x, y, z],
                    normal: [nx, ny, nz],
                    uv: [u, v],
                });

                // Update bounding box
                bounding_box.min.x = bounding_box.min.x.min(x);
                bounding_box.min.y = bounding_box.min.y.min(y);
                bounding_box.min.z = bounding_box.min.z.min(z);
                bounding_box.max.x = bounding_box.max.x.max(x);
                bounding_box.max.y = bounding_box.max.y.max(y);
                bounding_box.max.z = bounding_box.max.z.max(z);
            }

            // Push indices
            indices.extend_from_slice(&mesh.indices);
        }

        // Return the mesh
        Ok(MeshAsset {
            label,
            vertices, indices, bounding_box
        })
    }

    fn extensions(&self) -> &[&str] {
        &["obj", "fbx"]
    }
}



pub struct GpuMesh {
    /// The label of the mesh
    pub label: String,
    /// The vertex buffer
    pub vertex_buffer: WBuffer,
    /// The index buffer
    pub index_buffer: WBuffer,
    /// The number of indices
    pub index_count: u32,
    /// The bounding box of the model
    pub bounding_box: ModelBoundingBox,
}
impl RenderAsset for GpuMesh {
    type SourceAsset = MeshAsset;
    type Param = SRes<WRenderInstance<'static>>;

    fn prepare_asset(
            asset: Self::SourceAsset,
            render_instance: &mut bevy::ecs::system::SystemParamItem<Self::Param>,
        ) -> Result<Self, PrepareAssetError<Self::SourceAsset>> {
        debug!(asset.label, "Loading mesh on the GPU.");

        // Create vertex buffer
        let render_instance = render_instance.data.read().unwrap();
        let vertex_buffer = WBuffer::new(
            &render_instance,
            format!("{}-vertex", asset.label).as_str(),
            std::mem::size_of::<WVertex>() * asset.vertices.len(),
            BufferUsage::VERTEX,
            Some(bytemuck::cast_slice(&asset.vertices)));

        // Create index buffer
        let index_buffer = WBuffer::new(
            &render_instance,
            format!("{}-indices", asset.label).as_str(),
            std::mem::size_of::<u32>() * asset.indices.len(),
            BufferUsage::INDEX,
            Some(bytemuck::cast_slice(&asset.indices)));
        
        Ok(GpuMesh {
            label: asset.label,
            vertex_buffer,
            index_buffer,
            index_count: asset.indices.len() as u32,
            bounding_box: asset.bounding_box,
        })
    }

    fn label(&self) -> &str {
        &self.label
    }
}
