struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) view_ray: vec3<f32>,
};

struct Camera {
    world_to_ndc: mat4x4<f32>,
    ndc_to_world: mat4x4<f32>,
    position: vec4<f32>,
    z_near: f32,
    z_far: f32,
    padding: vec2<f32>,
}
@group(0) @binding(0) var<uniform> in_camera: Camera;

@group(1) @binding(0) var in_depth_texture: texture_depth_2d;
@group(1) @binding(1) var in_depth_sampler: sampler;

@group(2) @binding(0) var in_albedo_texture:   texture_2d<f32>;
@group(2) @binding(1) var in_albedo_sampler:   sampler;
@group(2) @binding(2) var in_normal_texture:   texture_2d<f32>;
@group(2) @binding(3) var in_normal_sampler:   sampler;
@group(2) @binding(4) var in_material_texture: texture_2d<f32>;
@group(2) @binding(5) var in_material_sampler: sampler;


@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Pbr material properties
    let ambient_strength = 0.01;

    // Light properties
    let light_color    = vec3<f32>(1.0, 1.0, 1.0);
    let light_position = vec3<f32>(5.0, 10.0, 5.0);

    // Read position of the object in world space
    let depth = textureSample(in_depth_texture, in_depth_sampler, in.tex_coord);
    let distance = in_camera.z_near * in_camera.z_far / (in_camera.z_far + depth * (in_camera.z_near - in_camera.z_far));
    let position = in_camera.position.xyz + normalize(in.view_ray) * distance;

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
    let light_dir = normalize(light_position - position);
    let light_angle = max(dot(normal.rgb, light_dir), 0.0);
    let diffused = light_angle * light_color;

    // Light transmitted by the light source through the material
    let transmitted = (ambient + diffused) * albedo.rgb;

    // Return the final color
    return vec4<f32>(transmitted, 1.0);
}
