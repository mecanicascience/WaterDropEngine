use bevy::prelude::*;

use super::TransformComponent;

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct CameraViewComponent {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Bundle, Default, Clone, Copy, Debug)]
pub struct CameraComponent {
    pub transform: TransformComponent,
    pub view: CameraViewComponent,
}