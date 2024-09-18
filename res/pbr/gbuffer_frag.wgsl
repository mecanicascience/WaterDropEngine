struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord:    vec2<f32>,
    @location(1) normal_world: vec3<f32>  // Normal in world space
};

struct FragOutput {
    @location(0) albedo:   vec4<f32>,
    @location(1) normal:   vec4<f32>,
    @location(2) material: vec4<f32>
};

// Material description
struct PbrMaterial {
    flags:    vec4<f32>, // x: has_albedo, y: has_specular
    albedo:   vec4<f32>,
    specular: f32
};
@group(2) @binding(0) var<uniform> in_material: PbrMaterial;
@group(2) @binding(1) var in_albedo_texture: texture_2d<f32>;
@group(2) @binding(2) var in_albedo_sampler: sampler;
@group(2) @binding(3) var in_specular_texture: texture_2d<f32>;
@group(2) @binding(4) var in_specular_sampler: sampler;

@fragment
fn main(in: VertexOutput) -> FragOutput {
    var out: FragOutput;
    
    // Read textures using material flags
    if (in_material.flags.x == 1.0) {
        out.albedo = textureSample(in_albedo_texture, in_albedo_sampler, in.tex_coord);
    } else {
        out.albedo = in_material.albedo;
    }
    if (in_material.flags.y == 1.0) {
        let specular_intensity = textureSample(in_specular_texture, in_specular_sampler, in.tex_coord).r;
        out.normal = vec4<f32>(normalize(in.normal_world), specular_intensity);
    } else {
        out.normal = vec4<f32>(normalize(in.normal_world), in_material.specular);
    }
    out.material = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    return out;
}
