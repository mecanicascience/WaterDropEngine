/// Structure for a bind group.
pub struct BindGroup {
    pub label: String,
    group: wgpu::BindGroup
}

impl BindGroup {
    /// Create a new bind group.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the bind group.
    /// * `group` - The group of the bind group.
    pub fn new(label: String, group: wgpu::BindGroup) -> Self {
        Self {
            label,
            group
        }
    }

    /// Get the group of the bind group.
    /// 
    /// # Returns
    /// 
    /// * `&wgpu::BindGroup` - The group of the bind group.
    pub fn get_group(&self) -> &wgpu::BindGroup {
        &self.group
    }
}
