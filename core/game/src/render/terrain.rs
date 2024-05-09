use tracing::error;

use wde_resources::{ResourcesManager, ShaderResource};
use wde_wgpu::{BindGroup, BindGroupBuilder, Buffer, BufferUsage, Color, CommandBuffer, LoadOp, Operations, RenderInstance, RenderPipeline, RenderTexture, ShaderType, StoreOp, Texture, Vertex};

use crate::{GameRenderPass, Scene};

/// The terrain renderer.
#[derive(Debug)]
pub struct TerrainRenderer {
    // Terrain data
    terrain_vertices: Buffer,
    terrain_indices: Buffer,
    terrain_indices_count: u32,
    terrain_pipeline: RenderPipeline,
}

impl TerrainRenderer {
    #[tracing::instrument(skip(camera_buffer_bg_build))]
    pub async fn new(camera_buffer_bg_build: BindGroupBuilder<'_>, render_instance: &RenderInstance<'_>, scene: &Scene, res_manager: &mut ResourcesManager) -> TerrainRenderer {
        // Terrain configuration
        const SUBDIVISION_COUNT: u32 = 128;
        let (terrain_width, terrain_height) = (10.0, 10.0);
        
        // Create terrain mesh
        let (terrain_vertices, terrain_indices, terrain_indices_count) = Self::create_terrain_mesh(
            render_instance, SUBDIVISION_COUNT, terrain_width, terrain_height);

        // Load terrain shaders
        let terrain_shader_vert = res_manager.load::<ShaderResource>("shaders/terrain/vert");
        res_manager.wait_for(&terrain_shader_vert, render_instance).await;
        let terrain_shader_frag = res_manager.load::<ShaderResource>("shaders/terrain/frag");
        res_manager.wait_for(&terrain_shader_frag, render_instance).await;

        // Create terrain description bind group
        // let mut objects_bind_group_layout = BindGroupBuilder::new("Objects matrices SSBO");
        // objects_bind_group_layout.add_buffer(0, &self.objects, ShaderStages::VERTEX, BufferBindingType::Storage { read_only: true });

        // Create terrain pipeline
        let mut terrain_pipeline = RenderPipeline::new("Terrain");
        let _ = terrain_pipeline
            .set_shader(&res_manager.get::<ShaderResource>(&terrain_shader_vert).unwrap().data.as_ref().unwrap().module, ShaderType::Vertex)
            .set_shader(&res_manager.get::<ShaderResource>(&terrain_shader_frag).unwrap().data.as_ref().unwrap().module, ShaderType::Fragment)
            .set_depth_stencil()
            .add_bind_group(BindGroup::new(&render_instance, camera_buffer_bg_build.clone()).layout)
            // .add_bind_group(BindGroup::new(&render_instance, objects_bind_group_layout.clone()).layout)
            .init(&render_instance).await
            .unwrap_or_else(|_| {
                error!("Failed to initialize terrain pipeline.");
            });
            
        TerrainRenderer {
            terrain_vertices,
            terrain_indices,
            terrain_indices_count,
            terrain_pipeline
        }
    }

    /// Create a terrain mesh.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    /// * `subdivisions` - The number of subdivisions
    /// * `width` - The width of the terrain
    /// * `height` - The height of the terrain
    pub fn create_terrain_mesh(render_instance: &RenderInstance, subdivisions: u32, width: f32, height: f32) -> (Buffer, Buffer, u32) {
        // Create vertices and indices
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        // Create vertices
        for i in 0..subdivisions {
            for j in 0..subdivisions {
                let x = i as f32 / subdivisions as f32 * width;
                let z = j as f32 / subdivisions as f32 * height;
                let y = 0.0;

                vertices.push(Vertex {
                    position: [x, y, z],
                    normal: [0.0, 1.0, 0.0],
                    tex_uv: [i as f32 / subdivisions as f32, j as f32 / subdivisions as f32],
                });
            }
        }

        // Create indices
        for i in 0..subdivisions - 1 {
            for j in 0..subdivisions - 1 {
                let a = i * subdivisions + j;
                let b = i * subdivisions + j + 1;
                let c = (i + 1) * subdivisions + j;
                let d = (i + 1) * subdivisions + j + 1;

                indices.push(a);
                indices.push(b);
                indices.push(c);

                indices.push(b);
                indices.push(d);
                indices.push(c);
            }
        }

        // Create vertex buffer
        let vertex_buffer = Buffer::new(
            &render_instance,
            format!("'{}' Vertex", "Terrain Mesh").as_str(),
            std::mem::size_of::<Vertex>() * vertices.len(),
            BufferUsage::VERTEX,
            Some(bytemuck::cast_slice(&vertices)));

        // Create index buffer
        let index_buffer = Buffer::new(
            &render_instance,
            format!("'{}' Index", "Terrain Mesh").as_str(),
            std::mem::size_of::<u32>() * indices.len(),
            BufferUsage::INDEX,
            Some(bytemuck::cast_slice(&indices)));

        return (vertex_buffer, index_buffer, indices.len() as u32);
    }
}

impl GameRenderPass for TerrainRenderer {
    // #[tracing::instrument]
    fn render(&self, command_buffer: &mut CommandBuffer, render_texture: &RenderTexture, depth_texture: &Texture, camera_buffer_bg: &BindGroup, _scene: &Scene, _res_manager: &mut ResourcesManager) {
        // Create render pass
        let mut render_pass = command_buffer.create_render_pass(
            "Terrain",
            &render_texture.view,
            Some(Operations {
                load: LoadOp::Clear(Color { r : 0.1, g: 0.105, b: 0.11, a: 1.0 }),
                store: StoreOp::Store,
            }),
            Some(&depth_texture.view)
        );

        // Set bind groups
        render_pass.set_bind_group(0, &camera_buffer_bg);
        // render_pass.set_bind_group(1, &self.terrain_bind_group);

        // Get terrain component
        render_pass.set_vertex_buffer(0, &self.terrain_vertices);
        render_pass.set_index_buffer(&self.terrain_indices);

        // Set pipeline
        if render_pass.set_pipeline(&self.terrain_pipeline).is_err() {
            error!("Failed to set terrain pipeline.");
            return;
        }

        // Draw
        render_pass
            .draw_indexed(0..self.terrain_indices_count, 0..1)
            .unwrap_or_else(|_| {
                error!("Failed to draw terrain.");
            });
    }

    fn label(&self) -> &str { "Terrain" }
}
