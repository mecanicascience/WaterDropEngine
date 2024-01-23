use tracing::{span, Level};
use wde_logger::{debug, trace, warn, error, throw, info};

use crate::{Window, TextureView};

/// Error type of the renderer.
#[derive(Debug)]
pub enum RenderError {
    /// Cannot present render texture.
    CannotPresent,
    /// Cannot resize render instance.
    CannotResize,
    /// Pipeline not set.
    PipelineNotSet,
    /// Pipeline not initialized.
    PipelineNotInitialized,
    /// Missing a shader.
    MissingShader,
    /// Missing a vertex buffer.
    MissingVertexBuffer,
    /// Missing an index buffer.
    MissingIndexBuffer,
    /// Swapchain format not supported.
    UnsupportedSwapchainFormat,
    /// Depth format not supported.
    UnsupportedDepthFormat,
}

/// Type of the render texture.
#[derive(Debug)]
pub struct RenderTexture {
    /// Texture of the render texture.
    pub texture: wgpu::SurfaceTexture,
    /// View of the render texture.
    pub view: TextureView,
}

/// Type of the render event.
#[derive(Debug)]
pub enum RenderEvent {
    /// Redraw the window.
    Redraw(RenderTexture),
    /// Close the window.
    Close,
    /// Resize the window.
    Resize(u32, u32),
    /// No event.
    None,
}

/// Instance of the GPU device required for the renderer.
/// 
/// # Example
/// 
/// ```
/// let mut instance = RenderInstance::new("WaterDropEngine", Some(&window)).await;
/// 
/// // Get current texture
/// let render_texture = instance.get_current_texture();
/// 
/// // Render to texture
/// {
///    // Create command buffer
///     let mut command_buffer = CommandBuffer::new(&instance, "Main render");
/// 
///     // Render
///     (...)
/// }
/// 
/// // Present texture
/// instance.present(render_texture);
/// ```
pub struct RenderInstance {
    /// Label of the instance.
    pub label: String,
    /// Instance of the GPU device.
    pub device: wgpu::Device,
    /// Queue for the GPU device.
    pub queue: wgpu::Queue,
    /// Surface of the GPU device.
    pub surface: Option<wgpu::Surface>,
    /// Surface configuration of the GPU device.
    pub surface_config: Option<wgpu::SurfaceConfiguration>,
}

impl std::fmt::Debug for RenderInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderInstance")
            .field("label", &self.label)
            .field("surface_config", &self.surface_config)
            .finish()
    }
}

impl RenderInstance {
    /// Create a new instance of the GPU device.
    /// 
    /// # Arguments
    /// 
    /// * `label` - Label of the instance.
    /// * `window` - Window of the instance. If `None`, the instance will be created without a surface.
    #[tracing::instrument]
    pub async fn new(label: &str, window: Option<&Window>) -> Self {
        info!(label, "Creating render instance.");

        // Create wgpu instance
        trace!(label, "Creating wgpu instance.");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN, // Ask for Vulkan backend
            flags: wgpu::InstanceFlags::DEBUG,
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        // Retrieve surface and adapter
        trace!(label, "Retrieving surface and adapter.");
        let window_ref = if window.is_some() {
            let window_instance = window.as_ref().unwrap();
            if window_instance.window.is_none() {
                throw!("Cannot create render instance without a window for '{}'.", label);
            }
            Some(window_instance.window.as_ref().unwrap())
        } else {
            None
        };
        let surface = if window.is_some() {
            Some(
                unsafe { instance.create_surface(&window_ref.unwrap()) }
                .unwrap_or_else(|e| throw!("Failed to create surface for '{}': {:?}.", label, e)))
        }
        else {
            None
        };
        let adaptater = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: match surface {
                    Some(ref s) => Some(s),
                    None => None
                },
                force_fallback_adapter: false,
            })
            .await
            .unwrap_or_else(|| throw!("Failed to create adapter for '{}'.", label));

        // Create device instance and queue
        trace!(label, "Requesting device.");
        let (device, queue) = adaptater
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some(label),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default()
                },
                None,
            )
            .await
            .unwrap_or_else(|e| throw!("Failed to create device for '{}': {:?}.", label, e));

        // If no surface, return instance
        if surface.is_none() {
            return RenderInstance {
                label: label.to_string(),
                device,
                queue,
                surface: None,
                surface_config: None,
            }
        }

        // Retrieve surface format (sRGB if possible)
        let surface_caps = surface.as_ref().unwrap().get_capabilities(&adaptater);
        let surface_format = surface_caps.formats.iter()
            .copied()
            .filter(|f| f.is_srgb()) 
            .next()
            .unwrap_or(surface_caps.formats[0]);

        // Set surface configuration
        trace!(label, "Configuring surface.");
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.unwrap().size.0,
            height: window.unwrap().size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![]
        };
        surface.as_ref().unwrap().configure(&device, &surface_config);

        // Return instance
        RenderInstance {
            label: label.to_string(),
            device,
            queue,
            surface,
            surface_config: Some(surface_config)
        }
    }

    /// Get the render texture.
    /// 
    /// # Arguments
    /// 
    /// * `render_instance` - Instance of the render.
    /// 
    /// # Returns
    /// 
    /// * `RenderEvent` - Render event.
    #[tracing::instrument]
    pub fn get_current_texture(render_instance: &RenderInstance) -> RenderEvent {
        // If no surface, return
        if render_instance.surface.is_none() {
            warn!(render_instance.label, "Cannot render to texture without a surface.");
            return RenderEvent::None;
        }

        // Get current texture
        debug!(render_instance.label, "Getting current texture.");
        let _get_current_texture = span!(Level::INFO, "acquire_texture").entered();
        let render_texture = render_instance.surface.as_ref().unwrap().get_current_texture();
        drop(_get_current_texture);

        // Check if texture is acquired
        trace!(render_instance.label, "Checking if texture is acquired.");
        match render_texture {
            Ok(surface_texture) => {
                // Create render view
                trace!(render_instance.label, "Creating render view.");
                let render_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Render Texture"),
                    format: match render_instance.surface_config.as_ref().unwrap().format {
                        wgpu::TextureFormat::Bgra8UnormSrgb => Some(wgpu::TextureFormat::Bgra8UnormSrgb),
                        wgpu::TextureFormat::Rgba8UnormSrgb => Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                        _ => throw!("Unsupported surface format for '{}'.", render_instance.label)
                    },
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                });
                let cur_render = RenderTexture {
                    texture: surface_texture,
                    view: render_view
                };
                return RenderEvent::Redraw(cur_render);
            }
            // Surface lost or outdated (minimized or moved to another screen)
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Resize surface
                let conf = render_instance.surface_config.as_ref().unwrap().clone();
                return RenderEvent::Resize(conf.width, conf.height);
            },
            // System out of memory
            Err(wgpu::SurfaceError::OutOfMemory) => {
                error!(render_instance.label, "System out of memory.");
                return RenderEvent::Close;
            }
            // Timeout of the surface
            Err(wgpu::SurfaceError::Timeout) => {
                warn!(render_instance.label, "Timeout of the surface.");
                return RenderEvent::None;
            }
        }
    }

    /// Present the render texture.
    /// This must be called after the render function.
    /// 
    /// # Arguments
    /// 
    /// * `render_texture` - Render texture to present.
    /// 
    /// # Errors
    /// 
    /// * `RenderError::CannotPresent` - Cannot present render texture.
    #[tracing::instrument]
    pub fn present(&self, render_texture: RenderTexture) -> Result<(), RenderError> {
        // If no surface, return
        if self.surface.is_none() {
            error!(self.label, "Cannot present render texture without a surface.");
            return Err(RenderError::CannotPresent);
        }

        // Present render texture
        debug!(self.label, "Presenting render texture.");
        render_texture.texture.present();
        Ok(())
    }

    /// Resize the surface of the instance.
    /// This must be called when the window is resized.
    /// 
    /// # Arguments
    /// 
    /// * `width` - New width of the surface.
    /// * `height` - New height of the surface.
    /// 
    /// # Errors
    /// 
    /// * `RenderError::CannotResize` - Cannot resize render instance surface.
    #[tracing::instrument]
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        trace!(self.label, width, height, "Resizing render instance.");
        // If no surface, return
        if self.surface.is_none() {
            error!(self.label, "Cannot resize render instance without a surface.");
            return Err(RenderError::CannotResize);
        }

        // Resize surface
        self.surface_config.as_mut().unwrap().width = width;
        self.surface_config.as_mut().unwrap().height = height;
        self.surface.as_ref().unwrap().configure(&self.device, &self.surface_config.as_ref().unwrap());
        Ok(())
    }
}

impl Drop for RenderInstance {
    #[tracing::instrument]
    fn drop(&mut self) {
        info!(self.label, "Dropping render instance.");
    }
}
