use wde_math::{Mat4f, Quatf, Vec3f};

/// Define the transform uniform buffer aligned to 16 bytes for the GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformUniform {
    /// From object to world space.
    object_to_world: [[f32; 4]; 4]
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
    pub fn new(transform: TransformComponent) -> Self {
        Self {
            object_to_world: TransformComponent::transform_obj_to_world(transform).into()
        }
    }
}



/// Store the position, rotation and scale of an entity.
#[derive(Debug, Copy, Clone)]
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
    /// The matrix transform from object to world space (translate * rotate * scale).
    #[tracing::instrument]
    pub fn transform_obj_to_world(transform: TransformComponent) -> Mat4f {
        // Compute rotation matrix components
        let q = transform.rotation;
        let x2 = q.v.x + q.v.x;
        let y2 = q.v.y + q.v.y;
        let z2 = q.v.z + q.v.z;

        let xx2 = x2 * q.v.x;
        let xy2 = x2 * q.v.y;
        let xz2 = x2 * q.v.z;

        let yy2 = y2 * q.v.y;
        let yz2 = y2 * q.v.z;
        let zz2 = z2 * q.v.z;

        let sy2 = y2 * q.s;
        let sz2 = z2 * q.s;
        let sx2 = x2 * q.s;

        // Compute composition with rotation
        let t = transform.position;
        let wx = (1.0 - yy2 - zz2) * t.x + (xy2 - sz2) * t.y + (xz2 + sy2) * t.z;
        let wy = (xy2 + sz2) * t.x + (1.0 - xx2 - zz2) * t.y + (yz2 - sx2) * t.z;
        let wz = (xz2 - sy2) * t.x + (yz2 + sx2) * t.y + (1.0 - xx2 - yy2) * t.z;

        // Compute scale
        let s = transform.scale;

        // Compute matrix
        Mat4f::new(
            (1.0 - yy2 - zz2) * s.x, xy2 + sz2, xz2 - sy2, 0.0,
            xy2 - sz2, (1.0 - xx2 - zz2) * s.y, yz2 + sx2, 0.0,
            xz2 + sy2, yz2 - sx2, (1.0 - xx2 - yy2) * s.z, 0.0,
            wx, wy, wz, 1.0
        )
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
    #[tracing::instrument]
    pub fn transform_world_to_obj(transform: TransformComponent) -> Mat4f {
        // Compute rotation matrix components
        let q = transform.rotation;
        let x2 = q.v.x + q.v.x;
        let y2 = q.v.y + q.v.y;
        let z2 = q.v.z + q.v.z;

        let xx2 = x2 * q.v.x;
        let xy2 = x2 * q.v.y;
        let xz2 = x2 * q.v.z;

        let yy2 = y2 * q.v.y;
        let yz2 = y2 * q.v.z;
        let zz2 = z2 * q.v.z;

        let sy2 = y2 * q.s;
        let sz2 = z2 * q.s;
        let sx2 = x2 * q.s;

        // Coeffs
        let a1 = 1.0 - yy2 - zz2;
        let a2 = xy2 + sz2;
        let a3 = xz2 - sy2;
        let a4 = xy2 - sz2;
        let a5 = 1.0 - xx2 - zz2;
        let a6 = yz2 + sx2;
        let a7 = xz2 + sy2;
        let a8 = yz2 - sx2;
        let a9 = 1.0 - xx2 - yy2;

        // Scale
        let s = transform.scale;

        // Compute det
        let det = a1*a5*a9 - a1*a6*a8 - a2*a4*a9 + a2*a6*a7 + a3*a4*a8 - a3*a5*a7;
        
        // Compute matrix
        Mat4f::new(
            (a5*a9 - a6*a8)/s.x/det, (-a2*a9 + a3*a8)/s.x/det, (a2*a6 - a3*a5)/s.x/det, 0.0,
            (-a4*a9 + a6*a7)/s.y/det, (a1*a9 - a3*a7)/s.y/det, (-a1*a6 + a3*a4)/s.y/det, 0.0,
            (a4*a8 - a5*a7)/s.z/det, (-a1*a8 + a2*a7)/s.z/det, (a1*a5 - a2*a4)/s.z/det, 0.0,
            -transform.position.x, -transform.position.y, -transform.position.z, 1.0
        )
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
