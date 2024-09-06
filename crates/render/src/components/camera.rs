use bevy::prelude::*;

use super::TransformUniform;

/// Camera view component with field of view, aspect ratio, near and far planes.
#[derive(Component, Clone, Debug)]
pub struct CameraView {
    pub fov: f32,
    pub znear: f32,
    pub zfar: f32,
}
impl Default for CameraView {
    fn default() -> Self {
        Self {
            fov: 60.0,
            znear: 0.1,
            zfar: 1000.0,
        }
    }
}

/// Camera bundle with a position and a view.
#[derive(Bundle, Default, Clone, Debug)]
pub struct Camera {
    pub transform: Transform,
    pub view: CameraView,
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
    /// * `aspect_ratio` - The aspect ratio of the screen.
    /// 
    /// # Returns
    /// 
    /// The world to screen (NDC) matrix ((openGL to WGPU) * projection * view).
    #[inline]
    pub fn get_world_to_ndc(transform: &Transform, camera_view: &CameraView, aspect_ratio: f32) -> Mat4 {
        // World to camera
        let view = TransformUniform::transform_world_to_obj(transform);

        // Projection from camera to NDC
        let proj = Mat4::perspective_rh(
            camera_view.fov.to_radians(), aspect_ratio,
            camera_view.znear, camera_view.zfar
        );
        proj * view
    }
}
