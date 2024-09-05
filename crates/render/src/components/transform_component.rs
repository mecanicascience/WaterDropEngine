use bevy::prelude::*;

/// Define the transform uniform buffer aligned to 16 bytes for the GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    /// From object to world space.
    pub object_to_world: [[f32; 4]; 4]
}

impl TransformUniform {
    /// Create a new transform uniform.
    /// 
    /// # Arguments
    /// 
    /// * `transform` - The transform component.
    /// 
    /// # Returns
    /// 
    /// The transform uniform.
    pub fn new(transform: &TransformComponent) -> Self {
        Self {
            object_to_world: TransformComponent::transform_obj_to_world(transform).to_cols_array_2d()
        }
    }
}



/// Store the position, rotation and scale of an entity.
#[derive(Component, Default, Clone, Debug)]
pub struct TransformComponent {
    /// The position of the entity.
    pub position: Vec3,
    /// The rotation of the entity.
    pub rotation: Quat,
    /// The scale of the entity.
    pub scale: Vec3,
}

impl TransformComponent {
    /// Get the matrix transform from object space to world space.
    /// 
    /// # Arguments
    /// 
    /// * `transform` - The transform component.
    /// 
    /// # Returns
    /// 
    /// The matrix transform from object to world space (translate * rotate * scale).
    pub fn transform_obj_to_world(transform: &TransformComponent) -> Mat4 {
        let translation = Mat4::from_translation(transform.position);
        let rotation = Mat4::from_quat(transform.rotation);
        let scale = Mat4::from_scale(transform.scale);

        translation * rotation * scale
    }

    /// Get the matrix transform from world space to object space.
    /// 
    /// # Arguments
    /// 
    /// * `transform` - The transform component.
    /// 
    /// # Returns
    /// 
    /// The matrix transform from world to object space (translate * rotate * scale)^(-1).
    pub fn transform_world_to_obj(transform: &TransformComponent) -> Mat4 {
        let translation = Mat4::from_translation(-transform.position);
        let rotation = Mat4::from_quat(transform.rotation).inverse();
        let scale = Mat4::from_scale(Vec3 {x: 1.0 / transform.scale.x, y: 1.0 / transform.scale.y, z: 1.0 / transform.scale.z});

        scale * rotation * translation
    }

    /// Get the forward vector (z axis) that the object is facing.
    /// 
    /// # Arguments
    /// 
    /// * `transform` - The transform component.
    pub fn forward(transform: TransformComponent) -> Vec3 {
        transform.rotation * Vec3 { x: 0.0, y: 0.0, z: -1.0 }
    }

    /// Get the right vector (x axis) that the object is facing.
    /// 
    /// # Arguments
    /// 
    /// * `transform` - The transform component.
    pub fn right(transform: TransformComponent) -> Vec3 {
        transform.rotation * Vec3 { x: 1.0, y: 0.0, z: 0.0 }
    }

    /// Get the up vector (y axis) that the object is facing.
    /// 
    /// # Arguments
    /// 
    /// * `transform` - The transform component.
    pub fn up(transform: TransformComponent) -> Vec3 {
        transform.rotation * Vec3 { x: 0.0, y: 1.0, z: 0.0 }
    }
}
