use tracing::{debug, error};

use crate::{BindGroup, ComputePipeline, RenderError};



/// Create a compute pass instance.
/// 
/// # Example
/// 
/// ```
/// let mut compute_pass = ComputePass::new(...);
/// 
/// // Set the pipeline dependencies
/// compute_pass
///     .set_pipeline(&[...])
///     .set_push_constants(&[...])
///     .set_bind_group(0, &[...]);
/// 
/// // Run compute pass
/// compute_pass.dispatch(x: [...], y: [...], z: [...]);
/// ```
pub struct ComputePass<'a> {
    pub label: String,
    compute_pass: wgpu::ComputePass<'a>,
    pipeline_set: bool,
}

impl std::fmt::Debug for ComputePass<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputePass")
            .field("label", &self.label)
            .finish()
    }
}

impl<'a> ComputePass<'a> {
    /// Create a new compute pass.
    /// 
    /// # Arguments
    /// 
    /// * `label` - The label of the compute pass.
    /// * `compute_pass` - The compute pass to create.
    #[tracing::instrument]
    pub fn new(label: &str, compute_pass: wgpu::ComputePass<'a>) -> Self {
        debug!(label, "Creating compute pass.");

        Self {
            label: label.to_string(),
            compute_pass,
            pipeline_set: false
        }
    }

    /// Set the pipeline of the compute pass.
    /// The bind groups of the pipeline are also set.
    /// 
    /// # Arguments
    /// 
    /// * `pipeline` - The pipeline to set.
    /// 
    /// # Errors
    /// 
    /// * `RenderError::PipelineNotInitialized` - The pipeline is not initialized.
    pub fn set_pipeline(&mut self, pipeline: &'a ComputePipeline) -> Result<&mut Self, RenderError> {
        if pipeline.get_pipeline().is_none() {
            error!(pipeline.label, "Pipeline is not created yet.");
            return Err(RenderError::PipelineNotInitialized);
        }

        // Set pipeline
        self.compute_pass.set_pipeline(&pipeline.get_pipeline().as_ref().unwrap());
        self.pipeline_set = true;
        Ok(self)
    }



    /// Set push constants of the compute pass.
    /// 
    /// # Arguments
    /// 
    /// * `data` - The data to set.
    pub fn set_push_constants(&mut self, data: &[u8]) -> &mut Self {
        self.compute_pass.set_push_constants(0, data);
        self
    }

    /// Set a bind group of the compute pass at a binding.
    /// 
    /// # Arguments
    /// 
    /// * `binding` - The binding of the bind group.
    /// * `bind_group` - The bind group to set.
    pub fn set_bind_group(&mut self, binding: u32, bind_group: &'a BindGroup) -> &mut Self {
        self.compute_pass.set_bind_group(binding, &bind_group.group, &[]);
        self
    }


    /// Dispatch the compute pass.
    /// 
    /// # Arguments
    /// 
    /// * `x` - The x dimension.
    /// * `y` - The y dimension.
    /// * `z` - The z dimension.
    /// 
    /// # Errors
    /// 
    /// * `RenderError::PipelineNotSet` - The pipeline is not set.
    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) -> Result<(), RenderError> {
        if !self.pipeline_set {
            error!(self.label, "Pipeline is not set.");
            return Err(RenderError::PipelineNotSet);
        }

        // Dispatch
        self.compute_pass.dispatch_workgroups(x, y, z);
        Ok(())
    }
    
}

impl Drop for ComputePass<'_> {
    #[tracing::instrument]
    fn drop(&mut self) {
        debug!(self.label, "Dropping compute pass.");
    }
}
