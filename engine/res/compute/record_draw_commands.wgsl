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
@group(0) @binding(0)
var<storage, read> batches: array<IndirectBatch>;

struct DrawIndexedIndirectCommand {
    /// Number of indices to draw
    index_count: u32,
    /// Number of instances to draw
    instance_count: u32,
    /// The base index within the index buffer
    first_index: u32,
    /// The base vertex within the vertex buffer
    base_vertex: i32,
    /// The base instance within the instance buffer
    first_instance: u32,
};
@group(1) @binding(0)
var<storage, read_write> indirect_commands: array<DrawIndexedIndirectCommand>;


 
@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // Get the index of the batch
    let index = id.x;

    // Record the indirect command
    let batch = batches[index];
    let command = DrawIndexedIndirectCommand(
        batch.index_count,
        batch.count,
        0,
        0,
        batch.first);
    indirect_commands[index] = command;
}