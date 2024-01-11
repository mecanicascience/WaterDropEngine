use wde_logger::{debug, trace, warn, error, throw};

use crate::{Window, TextureView};

/// Type of the render texture.
pub struct RenderTexture {
    /// Texture of the render texture.
    pub texture: wgpu::SurfaceTexture,
    /// View of the render texture.
    pub view: TextureView,
}

/// Type of the render event.
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
    /// Instance of the GPU device.
    pub device: wgpu::Device,
    /// Queue for the GPU device.
    pub queue: wgpu::Queue,
    /// Surface of the GPU device.
    pub surface: Option<wgpu::Surface>,
    /// Surface configuration of the GPU device.
    pub surface_config: Option<wgpu::SurfaceConfiguration>,
}

impl RenderInstance {
    /// Create a new instance of the GPU device.
    /// 
    /// # Arguments
    /// 
    /// * `label` - Label of the instance.
    /// * `window` - Window of the instance. If `None`, the instance will be created without a surface.
    pub async fn new(label: &str, window: Option<&Window>) -> Self {
        debug!("Creating render instance '{}'.", label);

        // Create wgpu instance
        trace!("Creating wgpu instance.");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::empty(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        // Retrieve surface and adapter
        trace!("Retrieving surface and adapter.");
        let window_ref = if window.is_some() {
            Some(window.as_ref().unwrap().window.as_ref().unwrap())
        } else {
            None
        };
        let surface = if window.is_some() {
            Some(unsafe { instance.create_surface(&window_ref.unwrap()) }.unwrap())
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
            .unwrap();

        // Create device instance and queue
        trace!("Requesting device.");
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
            .unwrap();

        // If no surface, return instance
        if surface.is_none() {
            return RenderInstance {
                device,
                queue,
                surface: None,
                surface_config: None
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
        trace!("Configuring surface.");
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
            device,
            queue,
            surface,
            surface_config: Some(surface_config)
        }
    }

    /// Get the render texture.
    pub fn get_current_texture(&mut self) -> RenderEvent {
        // If no surface, return
        if self.surface.is_none() {
            warn!("Cannot render to texture without a surface.");
            return RenderEvent::None;
        }

        // Get current texture
        trace!("Getting current texture.");
        let render_texture = self.surface.as_ref().unwrap().get_current_texture();

        // Check if texture is acquired
        match render_texture {
            Ok(surface_texture) => {
                // Create render view
                trace!("Creating render view.");
                let render_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("Render Texture"),
                    format: match self.surface_config.as_ref().unwrap().format {
                        wgpu::TextureFormat::Bgra8UnormSrgb => Some(wgpu::TextureFormat::Bgra8UnormSrgb),
                        wgpu::TextureFormat::Rgba8UnormSrgb => Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                        _ => throw!("Unsupported surface format.")
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
                let conf = self.surface_config.as_mut().unwrap().clone();
                self.resize(conf.width, conf.height);
                return RenderEvent::Resize(conf.width, conf.height);
            },
            // System out of memory
            Err(wgpu::SurfaceError::OutOfMemory) => {
                error!("System out of memory.");
                return RenderEvent::Close;
            }
            // Timeout of the surface
            Err(wgpu::SurfaceError::Timeout) => {
                warn!("Timeout of the surface.");
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
    pub fn present(&self, render_texture: RenderTexture) {
        // If no surface, return
        if self.surface.is_none() {
            error!("Cannot present render texture without a surface.");
            return;
        }

        // Present render texture
        trace!("Presenting render texture.");
        render_texture.texture.present();
    }

    /// Resize the surface of the instance.
    /// This must be called when the window is resized.
    /// 
    /// # Arguments
    /// 
    /// * `width` - New width of the surface.
    /// * `height` - New height of the surface.
    fn resize(&mut self, width: u32, height: u32) {
        trace!("Resizing render instance to {}x{}.", width, height);

        // Resize surface
        self.surface_config.as_mut().unwrap().width = width;
        self.surface_config.as_mut().unwrap().height = height;
        self.surface.as_ref().unwrap().configure(&self.device, &self.surface_config.as_ref().unwrap());
    }
}