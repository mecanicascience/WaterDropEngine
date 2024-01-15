use wde_math::{Mat4f, Quatf, Vec3f, SquareMatrix};

/// Store the position, rotation and scale of an entity.
#[derive(Copy, Clone)]
pub struct TransformComponent {
    /// The position of the entity.
    pub position: Vec3f,
    /// The rotation of the entity.
    pub rotation: Quatf,
    /// The scale of the entity.
    pub scale: Vec3f,
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
    /// The matrix transform from object to world space.
    pub fn transform_obj_to_world(transform: TransformComponent) -> Mat4f {
        let translation = Mat4f::from_translation(transform.position);
        let rotation = Mat4f::from(transform.rotation);
        let scale = Mat4f::from_nonuniform_scale(transform.scale.x, transform.scale.y, transform.scale.z);

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
    /// The matrix transform from world to object space.
    pub fn transform_world_to_obj(transform: TransformComponent) -> Mat4f {
        let translation_inv = Mat4f::from_translation(-transform.position);
        let rotation_inv = Mat4f::from(transform.rotation).invert().unwrap();
        let scale_inv = Mat4f::from_nonuniform_scale(1.0 / transform.scale.x, 1.0 / transform.scale.y, 1.0 / transform.scale.z);

        scale_inv * rotation_inv * translation_inv
    }

    /// Get the forward vector (z axis) that the object is facing.
    /// 
    /// # Arguments
    /// 
    /// * `transform` - The transform component.
    pub fn forward(transform: TransformComponent) -> Vec3f {
        transform.rotation * Vec3f { x: 0.0, y: 0.0, z: -1.0 }
    }

    /// Get the right vector (x axis) that the object is facing.
    /// 
    /// # Arguments
    /// 
    /// * `transform` - The transform component.
    pub fn right(transform: TransformComponent) -> Vec3f {
        transform.rotation * Vec3f { x: 1.0, y: 0.0, z: 0.0 }
    }

    /// Get the up vector (y axis) that the object is facing.
    /// 
    /// # Arguments
    /// 
    /// * `transform` - The transform component.
    pub fn up(transform: TransformComponent) -> Vec3f {
        transform.rotation * Vec3f { x: 0.0, y: 1.0, z: 0.0 }
    }
}
