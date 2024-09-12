struct ModelInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) view_ray: vec3<f32>,
};

struct Camera {
    world_to_ndc: mat4x4<f32>,
    ndc_to_world: mat4x4<f32>,
    position: vec4<f32>,
    z_near: f32,
    z_far: f32,
    padding: vec2<f32>,
}
@group(0) @binding(0) var<uniform> in_camera: Camera;

@vertex
fn main(@builtin(instance_index) instance: u32, model: ModelInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = vec4<f32>(model.position, 1.0);
    out.tex_coord = vec2<f32>(model.tex_coord.x, 1.0 - model.tex_coord.y); // Flip Y

    // Calculate the view ray in world space
    var position = in_camera.ndc_to_world * vec4<f32>(model.position, 1.0);
    position /= position.w;
    out.view_ray = position.xyz - in_camera.position.xyz;
    
    return out;
}
