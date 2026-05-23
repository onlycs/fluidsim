use crate::prelude::*;

use std::ops::{Deref, DerefMut};
use wgpu::PowerPreference;
use winit::{dpi::PhysicalSize, window::Window};

pub struct WgpuData {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub window: Arc<Window>,
    pub config: wgpu::SurfaceConfiguration,
}

#[derive(Default)]
pub struct WgpuState(Option<WgpuData>);

impl WgpuState {
    pub fn uninit(&self) -> bool {
        self.0.is_none()
    }

    pub async fn init(&mut self, window: Window, window_size: Vec2) -> Result<(), RendererError> {
        info!("Initializing renderer");

        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());

        let window = Arc::new(window);

        let Vec2 { x: winx, y: winy } = window_size;

        #[allow(unused_must_use)]
        window.request_inner_size(PhysicalSize::new(winx as i32, winy as i32));

        let surface = instance.create_surface(Arc::clone(&window))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::TIMESTAMP_QUERY
                    | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let caps = surface.get_capabilities(&adapter);
        let selected_fmt = [wgpu::TextureFormat::Bgra8Unorm];

        let texture_fmt = caps
            .formats
            .iter()
            .find(|f| selected_fmt.contains(f))
            .ok_or_else(|| RendererError::NoTextureFormat {
                available: format!("{:?}", caps.formats),
            })?;

        let surface_cfg = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *texture_fmt,
            width: winx as u32,
            height: winy as u32,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 1,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_cfg);

        self.0 = Some(WgpuData {
            surface,
            device,
            queue,
            config: surface_cfg,
            window,
        });

        Ok(())
    }
}

impl Deref for WgpuState {
    type Target = WgpuData;

    #[track_caller]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref().unwrap_unchecked() }
    }
}

impl DerefMut for WgpuState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.0.as_mut().unwrap_unchecked() }
    }
}
