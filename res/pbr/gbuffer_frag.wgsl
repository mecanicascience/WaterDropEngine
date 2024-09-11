struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

struct FragOutput {
    @location(0) color: vec4<f32>,
    @location(1) normal: vec4<f32>,
};

// Material description
struct PbrMaterial {
    albedo: vec3<f32>,
    has_texture: f32,
};
@group(2) @binding(0) var<uniform> in_material: PbrMaterial;
@group(2) @binding(1) var in_material_texture: texture_2d<f32>;
@group(2) @binding(2) var in_material_sampler: sampler;

@fragment
fn main(in: VertexOutput) -> FragOutput {
    var out: FragOutput;
    
    if (in_material.has_texture > 0.0) {
        out.color = textureSample(in_material_texture, in_material_sampler, in.tex_coord);
    } else {
        out.color = vec4<f32>(in_material.albedo, 1.0);
    }
    out.normal = vec4<f32>(normalize(in.normal), 1.0);

    return out;
}

fn map(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}
