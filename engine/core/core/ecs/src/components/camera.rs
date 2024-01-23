use wde_math::{Mat4f, SquareMatrix};

use crate::TransformComponent;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4f = Mat4f::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

/// Camera uniform buffer.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // From world to NDC coordinates
    pub world_to_screen: [[f32; 4]; 4]
}

impl CameraUniform {
    /// Create a new camera uniform buffer.
    pub fn new() -> Self {
        Self {
            world_to_screen: Mat4f::identity().into()
        }
    }

    /// Get the world to screen matrix.
    /// 
    /// # Arguments
    /// 
    /// * `camera` - The camera component.
    /// * `transform` - The transform component.
    /// 
    /// # Returns
    /// 
    /// The world to screen matrix ((openGL to WGPU) * projection * view).
    #[tracing::instrument]
    pub fn get_world_to_screen(camera: CameraComponent, transform: TransformComponent) -> Mat4f {
        // World to camera
        let view = TransformComponent::transform_world_to_obj(transform);
        // Projection from camera to NDC
        let proj = wde_math::perspective(
            wde_math::Deg::<f32>(camera.fovy), camera.aspect, camera.znear, camera.zfar
        );
        // Convert from OpenGL to WGPU (-1.0 / 1.0 to 0.0 / 1.0)
        let ndc = OPENGL_TO_WGPU_MATRIX * proj * view;
        
        // Return matrix world to screen
        ndc.into()
    }
}



/// Store the camera data component.
#[derive(Debug, Copy, Clone)]
pub struct CameraComponent {
    // Camera projection
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32
}
