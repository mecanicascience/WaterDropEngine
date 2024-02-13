// Vectors
pub type Vec2f = cgmath::Vector2<f32>;
pub type Vec3f = cgmath::Vector3<f32>;
pub type Vec4f = cgmath::Vector4<f32>;
pub type Vec2i = cgmath::Vector2<i32>;
pub type Vec3i = cgmath::Vector3<i32>;
pub type Vec4i = cgmath::Vector4<i32>;
pub type Vec2u = cgmath::Vector2<u32>;
pub type Vec3u = cgmath::Vector3<u32>;
pub type Vec4u = cgmath::Vector4<u32>;

pub const ZERO_VEC2F: Vec2f = Vec2f { x: 0.0, y: 0.0 };
pub const ZERO_VEC3F: Vec3f = Vec3f { x: 0.0, y: 0.0, z: 0.0 };
pub const ZERO_VEC4F: Vec4f = Vec4f { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
pub const ZERO_VEC2I: Vec2i = Vec2i { x: 0, y: 0 };
pub const ZERO_VEC3I: Vec3i = Vec3i { x: 0, y: 0, z: 0 };
pub const ZERO_VEC4I: Vec4i = Vec4i { x: 0, y: 0, z: 0, w: 0 };
pub const ZERO_VEC2U: Vec2u = Vec2u { x: 0, y: 0 };
pub const ZERO_VEC3U: Vec3u = Vec3u { x: 0, y: 0, z: 0 };
pub const ZERO_VEC4U: Vec4u = Vec4u { x: 0, y: 0, z: 0, w: 0 };

pub const ONE_VEC2F: Vec2f = Vec2f { x: 1.0, y: 1.0 };
pub const ONE_VEC3F: Vec3f = Vec3f { x: 1.0, y: 1.0, z: 1.0 };
pub const ONE_VEC4F: Vec4f = Vec4f { x: 1.0, y: 1.0, z: 1.0, w: 1.0 };
pub const ONE_VEC2I: Vec2i = Vec2i { x: 1, y: 1 };
pub const ONE_VEC3I: Vec3i = Vec3i { x: 1, y: 1, z: 1 };
pub const ONE_VEC4I: Vec4i = Vec4i { x: 1, y: 1, z: 1, w: 1 };
pub const ONE_VEC2U: Vec2u = Vec2u { x: 1, y: 1 };
pub const ONE_VEC3U: Vec3u = Vec3u { x: 1, y: 1, z: 1 };
pub const ONE_VEC4U: Vec4u = Vec4u { x: 1, y: 1, z: 1, w: 1 };

// Redefine core functions
pub use cgmath::*;


// Matrices
pub type Mat4f = cgmath::Matrix4<f32>;
pub type Mat4i = cgmath::Matrix4<i32>;
pub type Mat4u = cgmath::Matrix4<u32>;
pub use cgmath::SquareMatrix;

// Quaternions
pub type Quatf = cgmath::Quaternion<f32>;
pub type Quati = cgmath::Quaternion<i32>;
pub type Quatu = cgmath::Quaternion<u32>;

pub const QUATF_IDENTITY: Quatf = Quatf { s: 1.0, v: ZERO_VEC3F };
pub const QUATI_IDENTITY: Quati = Quati { s: 1, v: ZERO_VEC3I };
pub const QUATU_IDENTITY: Quatu = Quatu { s: 1, v: ZERO_VEC3U };
