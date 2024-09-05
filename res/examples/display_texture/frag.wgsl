struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>
};

// The texture to display
@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sam: sampler;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let val = textureSample(texture, texture_sam, in.tex_coord).r;
    return vec4<f32>(vec3<f32>(val), 1.0);
}
