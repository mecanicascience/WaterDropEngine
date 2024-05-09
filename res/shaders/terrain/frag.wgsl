struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

// Heightmap
@group(2) @binding(0) var heightmap: texture_2d<f32>;
@group(2) @binding(1) var heightmap_sam: sampler;

// Texture
@group(3) @binding(0) var terrain_tex: texture_2d<f32>;
@group(3) @binding(1) var terrain_sam: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color: vec4<f32> = textureSample(terrain_tex, terrain_sam, in.tex_coord);

    let light_dir: vec3<f32> = normalize(vec3<f32>(0.4, 1.0, 0.0));
    let light_intensity: f32 = max(dot(in.normal, light_dir), 0.0) * 0.6;

    let light_color: vec3<f32> = vec3<f32>(1.0, 1.0, 1.0);
    let ambient_color: vec3<f32> = vec3<f32>(0.1, 0.1, 0.1);
    let color: vec3<f32> = tex_color.xyz * (light_color * light_intensity + ambient_color);
    return vec4<f32>(color, tex_color.w);
}
