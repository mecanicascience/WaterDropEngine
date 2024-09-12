struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>
};

@group(0) @binding(0) var in_depth_texture: texture_depth_2d;
@group(0) @binding(1) var in_depth_sampler: sampler;

@group(1) @binding(0) var in_albedo_texture:   texture_2d<f32>;
@group(1) @binding(1) var in_albedo_sampler:   sampler;
@group(1) @binding(2) var in_normal_texture:   texture_2d<f32>;
@group(1) @binding(3) var in_normal_sampler:   sampler;
@group(1) @binding(4) var in_material_texture: texture_2d<f32>;
@group(1) @binding(5) var in_material_sampler: sampler;


@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Pbr material properties
    let ambient_strength = 0.01;

    // Light properties
    let light_color    = vec3<f32>(1.0, 1.0, 1.0);
    let light_position = vec3<f32>(3.0, 4.0, 2.0);

    // Read position from depth buffer
    let depth = textureSample(in_depth_texture, in_depth_sampler, in.tex_coord);
    // let distance = z_near * z_far / (z_far + depth * (z_near - z_far));

    // Read G-Buffer
    let albedo   = textureSample(in_albedo_texture,   in_albedo_sampler,   in.tex_coord);
    let normal   = textureSample(in_normal_texture,   in_normal_sampler,   in.tex_coord);
    let material = textureSample(in_material_texture, in_material_sampler, in.tex_coord);
    let metallic    = material.r;
    let roughness   = material.g;
    let reflectance = material.b;

    // Ambient light (light that is scattered in the atmosphere / moon light / ...)
    let ambient = ambient_strength * light_color;

    // Diffused light (direct light from the light source)
    let light_dir = normalize(light_position);
    let light_angle = max(dot(normal.rgb, light_dir), 0.0);
    let diffused = light_angle * light_color;

    // Light transmitted by the light source through the material
    let transmitted = (ambient + diffused) * albedo.rgb;

    // Return the final color
    // return vec4<f32>(normal.rgb, 1.0);
    return vec4<f32>(vec3<f32>(1.0 - depth), 1.0);
}
