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

struct TerrainDescription {
    obj_to_world: mat4x4<f32>,
    chunk_size: vec2<f32>,
    height: f32,
    padding: f32,
    chunks: vec2<u32>,
    padding2: vec2<u32>,
}
@group(1) @binding(0)
var<uniform> in_terrain: TerrainDescription;

// Heightmap
@group(2) @binding(0) var heightmap: texture_2d<f32>;

@vertex
fn main(
    @builtin(instance_index) instance: u32,
    model: ModelInput
) -> VertexOutput {
    var out: VertexOutput;

    // Convert instance index to 2D position
    var chunk_index = vec2<u32>(instance % in_terrain.chunks.x, instance / in_terrain.chunks.x);
    var pos = model.position;
    pos.x = pos.x + f32(chunk_index.x) * in_terrain.chunk_size.x * 0.99;
    pos.z = pos.z + f32(chunk_index.y) * in_terrain.chunk_size.y * 0.99;
    var tex_coord = vec2<f32>(
        model.tex_coord.x / f32(in_terrain.chunks.x) + f32(chunk_index.x) / f32(in_terrain.chunks.x),
        model.tex_coord.y / f32(in_terrain.chunks.y) + f32(chunk_index.y) / f32(in_terrain.chunks.y)
    );

    // Sample heightmap
    var heightmap_size = textureDimensions(heightmap);
    var height = textureLoad(heightmap, vec2<i32>(i32(tex_coord.x * f32(heightmap_size.x)), i32(tex_coord.y * f32(heightmap_size.y))), 0).r;
    pos.y = pos.y + height * in_terrain.height;

    // Set vertex data
    out.clip_position = in_camera.world_to_screen
        * in_terrain.obj_to_world
        * vec4<f32>(pos, 1.0);
    out.tex_coord = tex_coord;
    out.normal = model.normal;

    return out;
}
