struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>
};

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.tex_coord, 0.0, 1.0);
}
