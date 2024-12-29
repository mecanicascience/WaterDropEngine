// Description of the chunks parameters
struct MarchingCubesDescription {
    translation:       vec4<f32>,   // Translation in world space of the current chunk (x, y, z, 0)
    chunk_length:      vec4<f32>,   // Length of the chunk
    chunk_sub_count:   vec4<u32>,   // Number of sub-chunks
    triangles_counter: u32,         // Counter of the triangles
    iso_level:         f32,         // Iso level
    padding:           vec2<f32>    // Padding
}
@group(0) @binding(0) var<storage> in_desc: MarchingCubesDescription;

// List of points of the marching cubes values
@group(1) @binding(0) var<storage, read_write> in_points: array<vec4<f32>>; // Points of the marching cubes (x, y, z, f(x, y, z))


@compute @workgroup_size(10, 10, 10)
fn main(@builtin(global_invocation_id) thread_id: vec3<u32>) {
    // Check if out of bounds
    if thread_id.x >= in_desc.chunk_sub_count.x || thread_id.y >= in_desc.chunk_sub_count.y || thread_id.z >= in_desc.chunk_sub_count.z {
        return;
    }

    // Get the position of the current point
    let position = vec3<f32>(
        in_desc.translation.x - in_desc.chunk_length.x / 2.0 + f32(thread_id.x) * in_desc.chunk_length.x / (f32(in_desc.chunk_sub_count.x) - 1.0),
        in_desc.translation.y - in_desc.chunk_length.y / 2.0 + f32(thread_id.y) * in_desc.chunk_length.y / (f32(in_desc.chunk_sub_count.y) - 1.0),
        in_desc.translation.z - in_desc.chunk_length.z / 2.0 + f32(thread_id.z) * in_desc.chunk_length.z / (f32(in_desc.chunk_sub_count.z) - 1.0)
    );

    // Perlin noise parameters
    var amplitude = 60.0;
    var frequency = 0.005;
    let ground_percent = 0.1;

    let octaves = 8;
    let persistence = 0.5;
    let lacunarity = 2.0;

    var height = 0.0;
    for (var i = 0; i < octaves; i = i + 1) {
        height = height + amplitude * simplex_noise(position * frequency);
        amplitude = amplitude * persistence;
        frequency = frequency * lacunarity;
    }
    let ground = position.y + ground_percent * in_desc.chunk_length.y;
    let value = ground + height;

    // Store the value
    in_points[index_from_coord(thread_id, vec3<u32>(in_desc.chunk_sub_count.xyz))] = vec4<f32>(position, value);
}

/**
 * Get the index of a point in the grid from its coordinates.
 * 
 * # Arguments
 * 
 * * `x` - The x coordinate.
 * * `y` - The y coordinate.
 * * `z` - The z coordinate.
 * * `chunk_sub_count` - The number of grid cells in each dimension.
 */
fn index_from_coord(coord: vec3<u32>, chunk_sub_count: vec3<u32>) -> u32 {
    return (coord.x * chunk_sub_count.y + coord.y) * chunk_sub_count.z + coord.z;
}


/**
* Generate simplex noise from a 3D position.
* 
* # Arguments
* 
* * `position` - The position in 3D space.
*/
fn simplex_noise(position: vec3<f32>) -> f32 {
    // First corner
    let i = floor(position + dot(position, vec3<f32>(1.0 / 3.0)));
    let x0 = position - i + dot(i, vec3<f32>(1.0 / 6.0));

    // Other corners
    let g = step(x0.yzx, x0.xyz);
    let l = 1.0 - g;
    let i1 = min(g, l.zxy);
    let i2 = max(g, l.zxy);

    let x1 = x0 - i1 + vec3<f32>(1.0 / 6.0);
    let x2 = x0 - i2 + vec3<f32>(1.0 / 3.0);
    let x3 = x0 - vec3<f32>(0.5);

    // Permutations
    let p = permute(permute(permute(i.z + vec4<f32>(0.0, i1.z, i2.z, 1.0)) + i.y + vec4<f32>(0.0, i1.y, i2.y, 1.0)) + i.x + vec4<f32>(0.0, i1.x, i2.x, 1.0));

    // Gradients
    let gx = fract(p / 7.0) * 2.0 - 1.0;
    let gy = fract(floor(p / 7.0) / 7.0) * 2.0 - 1.0;
    let gz = 1.0 - abs(gx) - abs(gy);

    let g0 = normalize(vec3<f32>(gx.x, gy.x, gz.x));
    let g1 = normalize(vec3<f32>(gx.y, gy.y, gz.y));
    let g2 = normalize(vec3<f32>(gx.z, gy.z, gz.z));
    let g3 = normalize(vec3<f32>(gx.w, gy.w, gz.w));

    // Compute noise
    let m = max(0.5 - vec4<f32>(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3)), vec4<f32>(0.0));
    let m3 = m * m * m;
    let m4 = m * m3;

    let px = vec4<f32>(dot(g0, x0), dot(g1, x1), dot(g2, x2), dot(g3, x3));
    let temp = -8.0 * m3 * px;
    let grad = m4.x * g0 + temp.x * x0 + m4.y * g1 + temp.y * x1 + m4.z * g2 + temp.z * x2 + m4.w * g3 + temp.w * x3;

    return 107.0 * dot(m4, px);
}

fn permute(x: vec4<f32>) -> vec4<f32> {
    let temp = (x * 34 + 1) * x;
    return temp - floor(temp / 289) * 289;
}
