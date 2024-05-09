// Shader input and output
struct ModelInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

// Data
struct Camera {
    world_to_screen: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> in_camera: Camera;

struct ObjectToWorld {
    obj_to_world: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> in_model: ObjectToWorld;

// Heightmap
@group(2) @binding(0) var heightmap: texture_2d<f32>;


@vertex
fn main(
    @builtin(instance_index) instance: u32,
    model: ModelInput
) -> VertexOutput {
    var out: VertexOutput;

    // Sample heightmap
    var pos = model.position;
    var heightmap_size = textureDimensions(heightmap);
    var heightmap_pixel = textureLoad(heightmap, vec2<i32>(
        i32(model.tex_coord.x * f32(heightmap_size.x)), i32(model.tex_coord.y * f32(heightmap_size.y))), 0);
    var height = f32(heightmap_pixel.r);
    pos.y = pos.y + height * 5.0;
    
    // Set vertex data
    out.clip_position = in_camera.world_to_screen
        * in_model.obj_to_world
        * vec4<f32>(pos, 1.0);
    out.tex_coord = model.tex_coord;
    out.normal = model.normal;

    return out;
}
