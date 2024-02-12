@group(0) @binding(0)
var<storage, read_write> data: array<u32>;

struct Description {
    elements_count: u32;
    pass_index: u32;
}
@group(1) @binding(0)
var<push_constant> desc: Description;


@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // Get the index of the element to sort
    let index = 2 * id.x + desc.pass_index;
    if (index >= desc.elements_count || index + 1 >= desc.elements_count) {
        return;
    }

    // Get left value
    let left_value = data[index];

    // Get right value
    let right_value = data[index + 1];

    // Compare and swap
    if (left_value > right_value) {
        data[index] = right_value;
        data[index + 1] = left_value;
    }
} 