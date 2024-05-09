use wde_ecs::{CameraComponent, CameraUniform, EntityIndex, TransformComponent, World};
use wde_resources::ResourcesManager;
use wde_wgpu::{BindGroup, BindGroupBuilder, Buffer, BufferBindingType, CommandBuffer, RenderEvent, RenderInstance, RenderTexture, ShaderStages, Texture, TextureUsages};

use crate::{GameRenderPass, Scene, TerrainRenderer};


pub struct Renderer {
    // Render passes
    passes: Vec<Box<dyn GameRenderPass>>,
    depth_texture: Texture,
    // Camera
    camera_buffer: Buffer,
    camera_buffer_bg: BindGroup,
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("Renderer");
        for pass in &self.passes {
            d.field("pass", &pass.label());
        }
        d.finish()
    }
}

impl Renderer {
    /// Create a new renderer instance.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    /// * `scene` - The scene to render
    /// * `res_manager` - The resources manager
    #[tracing::instrument]
    pub async fn new(render_instance: &RenderInstance<'_>, scene: &mut Scene, res_manager: &mut ResourcesManager) -> Self {
        // Create camera uniform buffer
        let camera_buffer = Buffer::new(
            &render_instance,
            "Camera buffer",
            std::mem::size_of::<CameraUniform>(),
            wde_wgpu::BufferUsage::UNIFORM | wde_wgpu::BufferUsage::COPY_DST,
            None);

        // Create camera buffer bind group
        let mut camera_buffer_bg_build = BindGroupBuilder::new("Camera buffer");
        camera_buffer_bg_build
            .add_buffer(0, &camera_buffer, ShaderStages::VERTEX, BufferBindingType::Uniform);
        let camera_buffer_bg = BindGroup::new(&render_instance, camera_buffer_bg_build.clone());


        // Create list of passes (Note : The passes are loaded in order of rendering)
        let mut passes: Vec<Box<dyn GameRenderPass>> = Vec::new();
        // First pass : Terrain
        passes.push(Box::new(TerrainRenderer::new(camera_buffer_bg_build.clone(), render_instance, &scene.world, res_manager).await));

        // Create depth texture
        let depth_texture = Texture::new(
            render_instance,
            "Main depth texture",
            (render_instance.surface_config.as_ref().unwrap().width, render_instance.surface_config.as_ref().unwrap().height),
            Texture::DEPTH_FORMAT,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING);


        // Create instance
        Self {
            passes,
            depth_texture,
            camera_buffer,
            camera_buffer_bg,
        }
    }

    /// Update the renderer instance.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    /// * `scene` - The scene to render
    /// * `res_manager` - The resources manager
    #[tracing::instrument]
    pub fn update(&mut self, render_instance: &RenderInstance<'_>, scene: &Scene, res_manager: &ResourcesManager) {
        self.update_camera(render_instance, &scene.world, scene.active_camera);
    }

    /// Render the renderer instance.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    /// * `scene` - The scene to render
    /// * `res_manager` - The resources manager
    /// * `render_texture` - The render texture to render to
    #[tracing::instrument]
    pub async fn render(&self, render_instance: &RenderInstance<'_>, scene: &Scene, res_manager: &mut ResourcesManager, render_texture: &RenderTexture) -> RenderEvent {
        // Create command buffer
        let mut command_buffer = CommandBuffer::new(
                &render_instance, "Render").await;

        for i in 0..self.passes.len() {
            let pass = &self.passes[i];

            // Render pass
            pass.render(&mut command_buffer, &render_texture, &self.depth_texture, &self.camera_buffer_bg, &scene, res_manager);
        }

        // Submit command buffer
        command_buffer.submit(&render_instance);

        // Return
        RenderEvent::None
    }



    /// Update the active camera.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - The render instance
    /// * `camera_buffer` - The camera buffer
    #[tracing::instrument]
    fn update_camera(&mut self, render_instance: &RenderInstance, world: &World, camera: EntityIndex) {
        // Update camera component
        let mut camera_component = world.get_component::<CameraComponent>(camera).unwrap().clone();
        let surface_config = render_instance.surface_config.as_ref().unwrap();
        camera_component.aspect = surface_config.width as f32 / surface_config.height as f32;

        // Create camera uniform
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.world_to_screen = CameraUniform::get_world_to_screen(
            camera_component,
            world.get_component::<TransformComponent>(camera).unwrap().clone()
        ).into();

        // Write camera buffer
        self.camera_buffer.write(&render_instance, bytemuck::cast_slice(&[camera_uniform]), 0);
    }



    #[tracing::instrument]
    pub fn resize(&mut self, render_instance: &RenderInstance<'_>, width: u32, height: u32) {
        // Recreate depth texture
        self.depth_texture = Texture::new(
            render_instance,
            "Main depth texture",
            (width, height),
            Texture::DEPTH_FORMAT,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING);
    }
}
