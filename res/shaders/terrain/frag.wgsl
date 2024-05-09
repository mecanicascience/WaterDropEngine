struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let height = in.tex_coord.y;
    let normal = in.normal;

    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.0));
    let light_intensity = max(dot(normal, light_dir), 0.0);

    let color = vec4<f32>(vec3<f32>(height), 1.0) * light_intensity;
    return color;
}
