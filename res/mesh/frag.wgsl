struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 0.5, 1.0));
    let ambiant = 0.003;

    let intensity = max(dot(in.normal, light_dir), 0.0) + ambiant;
    let color = vec4<f32>(in.tex_coord, 0.0, 1.0);

    return color * intensity;
}
