struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>
};

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light_dir = normalize(vec3<f32>(0.5, 0.5, 1.0));
    let ambiant = 0.03;

    let intensity = max(dot(in.normal, light_dir), 0.0) + ambiant;
    var color = vec4<f32>(0.3, 0.3, 0.8, 1.0);

    return vec4<f32>(color.xyz * intensity, 1.0);
}

fn map(x: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}
