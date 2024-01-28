struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Return a color based on the normal of the vertex.
    return vec4<f32>(in.normal * 0.5 + 0.5, 1.0);
}
