struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

// Material description
struct PbrMaterial {
    base_color: vec3<f32>,
    has_texture: f32,
};
@group(2) @binding(0) var<uniform> in_material: PbrMaterial;
@group(2) @binding(1) var in_material_texture: texture_2d<f32>;
@group(2) @binding(2) var in_material_sampler: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 0.5, 1.0));
    let ambiant = 0.01;

    let intensity = max(dot(in.normal, light_dir), 0.0) + ambiant;
    var color = vec4<f32>(in_material.base_color, 1.0);

    if (in_material.has_texture == 1.0) {
        let min = 0.2;
        let max = 0.9;
        color.r *= map(textureSample(in_material_texture, in_material_sampler, in.tex_coord).r, min, max, 0.0, 1.0);
        color.g *= map(textureSample(in_material_texture, in_material_sampler, in.tex_coord).g, min, max, 0.0, 1.0);
        color.b *= map(textureSample(in_material_texture, in_material_sampler, in.tex_coord).b, min, max, 0.0, 1.0);
    }

    return color * intensity;
}

fn map(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}
