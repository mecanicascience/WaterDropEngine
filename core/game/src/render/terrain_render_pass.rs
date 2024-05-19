use std::u32;

use tracing::error;

use wde_ecs::World;
use wde_resources::{ResourcesManager, ShaderResource, TextureResource};
use wde_wgpu::{BindGroup, BindGroupBuilder, Buffer, BufferBindingType, BufferUsage, Color, CommandBuffer, LoadOp, Operations, RenderInstance, RenderPipeline, RenderTexture, ShaderStages, ShaderType, StoreOp, Texture, Vertex};

use crate::{GameRenderPass, Scene, TerrainComponent, TransformComponent, TransformUniform};

// Number of subdivisions
const SUBDIVISIONS: u32 = 128;

// Size of the terrain
const WIDTH: f32 = 10.0;
const HEIGHT: f32 = 10.0;

/// The terrain description shader struct.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainDescription {
    /// From object to world space.
    object_to_world: [[f32; 4]; 4],
    /// Size of the terrain.
    size: [f32; 2],
    /// Height of the terrain.
    height: f32,
    /// Padding.
    _padding: f32,
    /// Number of chunks.
    chunks: [u32; 2],
    /// Padding.
    _padding2: [u32; 2],
}

/// The terrain renderer.
#[derive(Debug)]
pub struct TerrainRenderPass {
    // Chunk render data
    chunk_vertices: Buffer,
    chunk_indices: Buffer,
    chunk_indices_count: u32,

    // Terrain pipeline
    terrain_pipeline: RenderPipeline,

    // Terrain position data
    terrain_description: Buffer,
    terrain_buffer_bg: BindGroup,

    // Terrain images data
    terrain_heightmap_bg: BindGroup,
    terrain_texture_bg: BindGroup,
}

impl TerrainRenderPass {
    #[tracing::instrument(skip(camera_buffer_bg_build))]
    pub async fn new(camera_buffer_bg_build: BindGroupBuilder<'_>, render_instance: &RenderInstance<'_>, world: &World, res_manager: &mut ResourcesManager) -> TerrainRenderPass {
        // Create terrain mesh
        let (chunk_vertices, chunk_indices, chunk_indices_count) = Self::create_chunk_mesh(render_instance);
        
        // Create terrain transform buffer
        let terrain_description = Buffer::new(
            render_instance,
            "Terrain description",
            std::mem::size_of::<TerrainDescription>(),
            BufferUsage::UNIFORM | BufferUsage::COPY_DST,
            None);

        // Create terrain description bind group
        let mut terrain_buffer_bg_build = BindGroupBuilder::new("Terrain description");
        terrain_buffer_bg_build.add_buffer(0, &terrain_description, ShaderStages::VERTEX, BufferBindingType::Uniform);
        let terrain_buffer_bg = BindGroup::new(&render_instance, terrain_buffer_bg_build.clone());


        // Load terrain shaders
        let terrain_shader_vert = res_manager.load::<ShaderResource>("shaders/terrain/vert");
        res_manager.wait_for(&terrain_shader_vert, render_instance).await;
        let terrain_shader_frag = res_manager.load::<ShaderResource>("shaders/terrain/frag");
        res_manager.wait_for(&terrain_shader_frag, render_instance).await;
        

        // Load terrain heightmap
        let terrain_heightmap = res_manager.load::<TextureResource>("texture/terrain_heightmap");
        res_manager.wait_for(&terrain_heightmap, render_instance).await;

        // Load terrain texture
        let terrain_texture = res_manager.load::<TextureResource>("texture/terrain_texture");
        res_manager.wait_for(&terrain_texture, render_instance).await;


        // Create heightmap description bind group
        let mut terrain_heightmap_bg_build = BindGroupBuilder::new("Terrain heightmap");
        terrain_heightmap_bg_build.add_texture(
            0, &res_manager.get::<TextureResource>(&terrain_heightmap).unwrap().data.as_ref().unwrap().texture, ShaderStages::VERTEX | ShaderStages::FRAGMENT);
        let terrain_heightmap_bg = BindGroup::new(&render_instance, terrain_heightmap_bg_build.clone());

        // Create texture description bind group
        let mut terrain_texture_bg_build = BindGroupBuilder::new("Terrain texture");
        terrain_texture_bg_build.add_texture(
            0, &res_manager.get::<TextureResource>(&terrain_texture).unwrap().data.as_ref().unwrap().texture, ShaderStages::FRAGMENT);
        let terrain_texture_bg = BindGroup::new(&render_instance, terrain_texture_bg_build.clone());
        

        // Create terrain pipeline
        let mut terrain_pipeline = RenderPipeline::new("Terrain");
        let _ = terrain_pipeline
            .set_shader(&res_manager.get::<ShaderResource>(&terrain_shader_vert).unwrap().data.as_ref().unwrap().module, ShaderType::Vertex)
            .set_shader(&res_manager.get::<ShaderResource>(&terrain_shader_frag).unwrap().data.as_ref().unwrap().module, ShaderType::Fragment)
            .set_depth_stencil()
            .add_bind_group(BindGroup::new(&render_instance, camera_buffer_bg_build.clone()).layout)
            .add_bind_group(BindGroup::new(&render_instance, terrain_buffer_bg_build).layout)
            .add_bind_group(BindGroup::new(&render_instance, terrain_heightmap_bg_build).layout)
            .add_bind_group(BindGroup::new(&render_instance, terrain_texture_bg_build).layout)
            .init(&render_instance)
            .unwrap_or_else(|_| {
                error!("Failed to initialize terrain pipeline.");
            });
            
        TerrainRenderPass {
            chunk_vertices,
            chunk_indices,
            chunk_indices_count,

            terrain_pipeline,

            terrain_description,
            terrain_buffer_bg,

            terrain_heightmap_bg,
            terrain_texture_bg,
        }
    }

    /// Create a chunk mesh.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    pub fn create_chunk_mesh(render_instance: &RenderInstance) -> (Buffer, Buffer, u32) {
        // Create vertices and indices
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        // Create vertices
        for i in 0..SUBDIVISIONS {
            for j in 0..SUBDIVISIONS {
                let x = i as f32 / SUBDIVISIONS as f32 * WIDTH - WIDTH / 2.0;
                let z = j as f32 / SUBDIVISIONS as f32 * HEIGHT - HEIGHT / 2.0;
                let y = 0.0;

                vertices.push(Vertex {
                    position: [x, y, z],
                    normal: [0.0, 1.0, 0.0],
                    tex_uv: [i as f32 / SUBDIVISIONS as f32, j as f32 / SUBDIVISIONS as f32],
                });
            }
        }

        // Create indices
        for i in 0..SUBDIVISIONS - 1 {
            for j in 0..SUBDIVISIONS - 1 {
                let a = i * SUBDIVISIONS + j;
                let b = i * SUBDIVISIONS + j + 1;
                let c = (i + 1) * SUBDIVISIONS + j;
                let d = (i + 1) * SUBDIVISIONS + j + 1;

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
            format!("'{}' Vertex", "Chunk vertices").as_str(),
            std::mem::size_of::<Vertex>() * vertices.len(),
            BufferUsage::VERTEX,
            Some(bytemuck::cast_slice(&vertices)));

        // Create index buffer
        let index_buffer = Buffer::new(
            &render_instance,
            format!("'{}' Index", "Chunk indices").as_str(),
            std::mem::size_of::<u32>() * indices.len(),
            BufferUsage::INDEX,
            Some(bytemuck::cast_slice(&indices)));

        return (vertex_buffer, index_buffer, indices.len() as u32);
    }
}

impl GameRenderPass for TerrainRenderPass {
    #[tracing::instrument]
    fn render(&mut self, render_instance: &RenderInstance, command_buffer: &mut CommandBuffer, render_texture: &RenderTexture, depth_texture: &Texture, camera_buffer_bg: &BindGroup, scene: &Scene, _res_manager: &mut ResourcesManager) {
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

        // Set global bind groups
        render_pass.set_bind_group(0, &camera_buffer_bg);

        // Get terrain component
        render_pass.set_vertex_buffer(0, &self.chunk_vertices);
        render_pass.set_index_buffer(&self.chunk_indices);
        
        // Render terrains
        for entity in scene.world.get_entities_with_component::<TerrainComponent>() {
            let terrain_component = scene.world.get_component::<TerrainComponent>(entity).unwrap();
            let transform_component = scene.world.get_component::<TransformComponent>(entity).unwrap();

            // Update terrain transform buffer
            let terrain_desc = TerrainDescription {
                object_to_world: TransformUniform::new(transform_component).object_to_world,
                chunks: [terrain_component.chunks.0, terrain_component.chunks.1],
                size: [WIDTH, HEIGHT],
                height: terrain_component.height,
                _padding: 0.0,
                _padding2: [0; 2],
            };
            self.terrain_description.write(render_instance, bytemuck::cast_slice(&[terrain_desc]), 0);

            // Set terrain bind groups
            render_pass.set_bind_group(1, &self.terrain_buffer_bg);
            render_pass.set_bind_group(2, &self.terrain_heightmap_bg);
            render_pass.set_bind_group(3, &self.terrain_texture_bg);

            // Set pipeline
            if render_pass.set_pipeline(&self.terrain_pipeline).is_err() {
                error!("Failed to set terrain pipeline.");
                return;
            }

            // Draw
            render_pass
                .draw_indexed(0..self.chunk_indices_count, 0..terrain_component.chunks.0 * terrain_component.chunks.1)
                .unwrap_or_else(|_| {
                    error!("Failed to draw terrain.");
                });
        }
    }

    fn label(&self) -> &str { "Terrain" }
}
