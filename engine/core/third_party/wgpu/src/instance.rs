use wde_logger::{debug, trace};

use crate::Window;

/// Instance of the GPU device required for the renderer.
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
}