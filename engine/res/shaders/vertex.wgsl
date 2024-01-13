struct ModelInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn main(
    @builtin(instance_index) instance: u32,
    model: ModelInput
) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = mat4x4<f32>(
        0.3000, 0.0000, 0.0000, 0.0000,
        0.0000, 0.3000, 0.0000, 0.0000,
        0.0000, 0.0000, 0.3000, 0.0000,
        -0.5000, 0.0000, 0.0000, 1.0000
    );
    let camera_matrix = mat4x4<f32>(
        0.9308, -0.0340, -0.2949, -0.2948,
        0.0000, 1.7282, -0.0665, -0.0665,
        -0.2879, -0.1100, -0.9533, -0.9532,
        0.2797, -1.3955, 2.9978, 3.0975
    );
    
    out.clip_position = camera_matrix
        * model_matrix
        * vec4<f32>(model.position, 1.0);
    out.tex_coord = model.tex_coord;
    out.normal = model.normal;

    return out;
}
