struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@group(0) @binding(0) var in_albedo_texture: texture_2d<f32>;
@group(0) @binding(1) var in_albedo_sampler: sampler;
@group(0) @binding(2) var in_normal_texture: texture_2d<f32>;
@group(0) @binding(3) var in_normal_sampler: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(in_albedo_texture, in_albedo_sampler, in.tex_coord);
    let normal = textureSample(in_normal_texture, in_normal_sampler, in.tex_coord);

    return vec4<f32>(normal.xyz, 1.0);
}
