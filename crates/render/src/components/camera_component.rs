use bevy::prelude::*;

use super::TransformComponent;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
]);

#[derive(Component, Clone, Debug)]
pub struct CameraViewComponent {
    pub fov: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
}
impl Default for CameraViewComponent {
    fn default() -> Self {
        Self {
            fov: 60.0,
            aspect: 1.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }
}

#[derive(Bundle, Default, Clone, Debug)]
pub struct CameraComponent {
    pub transform: TransformComponent,
    pub view: CameraViewComponent,
}

/// Camera uniform buffer.
#[repr(C)]
#[derive(Resource, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // From world to NDC coordinates
    pub world_to_ndc: [[f32; 4]; 4]
}

impl CameraUniform {
    /// Get the world to ndc matrix.
    /// 
    /// # Arguments
    /// 
    /// * `camera` - The camera component.
    /// * `transform` - The transform component.
    /// 
    /// # Returns
    /// 
    /// The world to screen (NDC) matrix ((openGL to WGPU) * projection * view).
    pub fn get_world_to_ndc(transform: &TransformComponent, camera_view: &CameraViewComponent) -> Mat4 {
        // World to camera
        let view = TransformComponent::transform_world_to_obj(transform);
        // Projection from camera to NDC
        let proj = Mat4::perspective_rh_gl(
            camera_view.fov, camera_view.aspect, camera_view.znear, camera_view.zfar
        );
        // Convert from OpenGL to WGPU (-1.0 / 1.0 to 0.0 / 1.0)
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}
