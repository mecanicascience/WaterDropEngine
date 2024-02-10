struct DrawIndirectCommandDescriptor {
    /// The offset to the first draw call
    first: u32,
    /// The number of draw calls
    count: u32,
    /// The index of the batch
    batch_index: u32,
    /// Padding
    _padding: u32,
};
@group(0) @binding(0)
var<storage, read> indirect_desc_temporary: array<DrawIndirectCommandDescriptor>;
@group(1) @binding(0)
var<storage, read_write> indirect_desc: array<DrawIndirectCommandDescriptor>;

struct OutputData {
    /// The number of descriptors that will generate indirect commands
    descriptor_count: atomic<u32>,
};
@group(2) @binding(0)
var<storage, read_write> output: OutputData;


@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // If same batch index as right neighbor, do nothing
    let curbatch = indirect_desc_temporary[id.x];
    if (curbatch.batch_index == indirect_desc_temporary[id.x + 1].batch_index) {
        return;
    }

    // Find first and count by looking at previous commands with same batch index
    var it = id.x;
    var first = curbatch.first;
    var count = 1;
    while (it > 0 && indirect_desc_temporary[it - 1].batch_index == curbatch.batch_index) {
        it = it - 1;
        first = indirect_desc_temporary[it].first;
        count = count + 1;
    }

    // Set the command
    let desc = DrawIndirectCommandDescriptor(
        u32(first),
        u32(count),
        curbatch.batch_index,
        0);
    indirect_desc[curbatch.batch_index] = desc;
    atomicAdd(&output.descriptor_count, u32(1));
}