struct ModelInput {
    @location(0) position:  vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal:    vec3<f32>
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord:    vec2<f32>,
    @location(1) normal_world: vec3<f32>, // Normal in world space
};

// From world space to normalized device coordinates
struct Camera {
    world_to_ndc: mat4x4<f32>
}
@group(0) @binding(0) var<uniform> in_camera: Camera;

// Object to world space transformation ssbo
struct ObjectToWorld {
    obj_to_world:  mat4x4<f32>
}
struct Model {
    data: array<ObjectToWorld>,
}
@group(1) @binding(0) var<storage> in_model: Model;


@vertex
fn main(@builtin(instance_index) instance: u32, model: ModelInput) -> VertexOutput {
    var out: VertexOutput;

    let obj_to_world = in_model.data[instance].obj_to_world;
    out.clip_position = in_camera.world_to_ndc
        * obj_to_world
        * vec4<f32>(model.position, 1.0);
    out.tex_coord = model.tex_coord;

    // Only works for uniform scaling
    let normal_matrix = mat3x3<f32>(obj_to_world[0].xyz, obj_to_world[1].xyz, obj_to_world[2].xyz);
    out.normal_world = normal_matrix * model.normal;

    return out;
}
