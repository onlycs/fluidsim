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

    pub async fn init(
        &mut self,
        window: Window,
        instance: &wgpu::Instance,
        window_size: Vec2,
    ) -> Result<(), RendererError> {
        info!("Initializing renderer");

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
            .await;

        let Some(adapter) = adapter else {
            return Err(RendererError::NoAdapter);
        };

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    #[cfg(not(target_arch = "wasm32"))]
                    required_limits: wgpu::Limits::default(),
                    #[cfg(target_arch = "wasm32")]
                    required_limits: wgpu::Limits {
                        max_storage_buffer_binding_size: 134217728,
                        max_compute_workgroup_storage_size: 17408,
                        ..wgpu::Limits::downlevel_defaults()
                    },
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await?;

        let caps = surface.get_capabilities(&adapter);
        let selected_fmt = [wgpu::TextureFormat::Rgba8Unorm];

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

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::prelude::Closure;
            use wasm_bindgen::JsCast;
            use web_sys::Event;
            use winit::platform::web::WindowExtWebSys;

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dest = doc.get_element_by_id("fluidsim")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dest.append_child(&canvas).ok()?;
                    Some(doc)
                })
                .and_then(|doc| {
                    let canvas = doc.query_selector("canvas").ok()??;
                    let help = doc.get_element_by_id("onboard-help")?;
                    help.set_attribute("style", "display: block;").ok()?;

                    let onclick = Closure::new(Box::new(move |_| {
                        doc.query_selector("canvas")
                            .unwrap()
                            .unwrap()
                            .set_attribute("style", "width: 100vw; height: 100vh;")
                            .unwrap();

                        help.set_attribute("style", "display: none;").ok().unwrap();
                    }) as Box<dyn FnMut(Event)>);

                    canvas
                        .add_event_listener_with_callback("click", onclick.as_ref().unchecked_ref())
                        .ok()?;

                    onclick.forget();

                    Some(())
                })
                .unwrap();
        }

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
