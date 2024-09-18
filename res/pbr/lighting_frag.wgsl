struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>
};

struct Camera {
    world_to_ndc: mat4x4<f32>,
    ndc_to_world: mat4x4<f32>,
    position: vec4<f32>
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

struct Light {
    /// World space position of the directional light for xyz. If it is the first element, the w component is the number of lights.
    position_type: vec4<f32>,
    /// World space direction of the light. The w component is the type of the light: 0 for directional, 1 for point, 2 for spot.
    direction:     vec4<f32>,
    /// Ambient color of the light. The w component is the constant attenuation factor if the light is a point light. It is the inner cut-off angle in radians if the light is a spot light.
    ambient_const_inn: vec4<f32>,
    /// Diffuse color of the light. The w component is the linear attenuation factor if the light is a point light. It is the outer cut-off angle in radians if the light is a spot light.
    diffuse_linea_out: vec4<f32>,
    /// Specular color of the light. The w component is the quadratic attenuation factor if the light is a point light.
    specular_quadr:    vec4<f32>
};
@group(3) @binding(0) var<storage> in_lights: array<Light>;



fn world_from_screen_coord(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let ndc_position   = vec4<f32>(uv.x * 2.0 - 1.0, (1 - uv.y) * 2.0 - 1.0, depth, 1.0);
    let view_position  = in_camera.ndc_to_world * ndc_position;
    let world_position = view_position.xyz / view_position.w;
    return world_position;
}

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Pbr material properties
    let shininess = 32.0;

    // Light properties
    let light_position = vec3<f32>(1.2, 1.0, 2.0);
    let light_ambiant  = vec3<f32>(0.2, 0.2, 0.2);
    let light_diffuse  = vec3<f32>(0.5, 0.5, 0.5);
    let light_specular = vec3<f32>(1.0, 1.0, 1.0);

    // Read position of the object in world space
    let depth = textureSample(in_depth_texture, in_depth_sampler, in.tex_coord);
    if depth == 1.0 { // Discard background
        discard;
    }
    let position = world_from_screen_coord(in.tex_coord, depth);

    // Read G-Buffer
    let g_albedo   = textureSample(in_albedo_texture,   in_albedo_sampler,   in.tex_coord).xyz;
    let g_nor_raw  = textureSample(in_normal_texture,   in_normal_sampler,   in.tex_coord);
    let g_normal   = g_nor_raw.xyz;
    let g_specular = g_nor_raw.w;
    let g_material = textureSample(in_material_texture, in_material_sampler, in.tex_coord);

    // General computed values
    let light_dir = normalize(light_position - position);

    // Ambient light (light that is scattered in the atmosphere / moon light / ...)
    let ambient = g_albedo * light_ambiant;

    // Diffused light (direct light from the light source, scattered by the material)
    let light_angle = max(dot(g_normal, light_dir), 0.0);
    let diffused    = (g_albedo * light_angle) * light_diffuse;

    // Specular light (light that is reflected by the material directly to the camera)
    let view_dir    = normalize(in_camera.position.xyz - position);
    let reflect_dir = reflect(-light_dir, g_normal);
    let spec_value  = pow(max(dot(view_dir, reflect_dir), 0.0), shininess);
    let specular    = (g_specular * spec_value) * light_specular;

    // Light transmitted by the light source through the material
    let transmitted = ambient + diffused + specular;

    // Read the number of lights
    let num_lights = i32(in_lights[0].position_type.w);

    // Return the final color
    return vec4<f32>(transmitted, 1.0);
}
