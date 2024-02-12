@group(0) @binding(0)
var<storage, read_write> batches_indices: array<u32>;

struct IndirectBatch {
    /// First entity index (Note that the index need to be the same as the index in the SSBO)
    first: u32,
    /// Number of entities
    count: u32,
    /// Number of indices in the model
    index_count: u32,
    /// Batch index. This uniquely identifies a model and material pair.
    batch_index: u32,
}
@group(0) @binding(1)
var<storage, read> batches: array<IndirectBatch>;

struct Description {
    /// The number of elements to sort
    elements_count: u32,
    /// The current pass index (0 or 1)
    pass_index: u32
}
var<push_constant> desc: Description;


@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // Get the index of the element to sort
    let index = 2 * id.x + desc.pass_index;
    if (index >= desc.elements_count || index + 1 >= desc.elements_count) {
        return;
    }

    // Get left value
    let left_index = batches_indices[index];
    let left_value = batches[left_index].batch_index;

    // Get right value
    let right_index = batches_indices[index + 1];
    let right_value = batches[right_index].batch_index;

    // Compare and swap
    if (left_value > right_value) {
        batches_indices[index] = right_index;
        batches_indices[index + 1] = left_index;
    }
} 