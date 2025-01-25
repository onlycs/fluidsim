use crate::prelude::*;

use shader::VsState;
use state::Game;
use std::ops::{Deref, DerefMut};
use wgpu::PowerPreference;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

mod shader;
mod state;

pub struct WgpuData {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    config: wgpu::SurfaceConfiguration,
}

pub struct WgpuState(Option<WgpuData>);

impl WgpuState {
    pub fn uninit(&self) -> bool {
        self.0.is_none()
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

pub struct SimRenderer {
    instance: wgpu::Instance,
    wgpu: WgpuState,
    shader: VsState,

    game: state::Game,
}

impl SimRenderer {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

        Self {
            instance,
            wgpu: WgpuState(None),
            game: Game::new(),
            shader: VsState::default(),
        }
    }

    async fn init_static(
        window: Window,
        instance: &wgpu::Instance,
        window_size: Vec2,
    ) -> Result<WgpuData, RendererError> {
        info!("Initializing renderer");

        let window = Arc::new(window);

        let Vec2 { x: winx, y: winy } = window_size;

        #[allow(unused_must_use)]
        window.request_inner_size(PhysicalSize::new(winx as i32, winy as i32));

        let surface = instance.create_surface(Arc::clone(&window))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: PowerPreference::HighPerformance,
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
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await?;

        let caps = surface.get_capabilities(&adapter);
        let selected_fmt = [
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
        ];

        let texture_fmt = caps
            .formats
            .iter()
            .find(|f| selected_fmt.contains(f))
            .unwrap_or_else(|| {
                info!("{:?}", caps.formats);
                panic!()
            });

        let surface_cfg = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *texture_fmt,
            width: winx as u32,
            height: winy as u32,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_cfg);

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dest = doc.get_element_by_id("fluidsim")?;
                    let canvas = web_sys::Element::from(window.canvas()?);
                    dest.append_child(&canvas).ok()?;
                    Some(())
                })
                .unwrap();
        }

        Ok(WgpuData {
            surface,
            device,
            queue,
            config: surface_cfg,
            window,
        })
    }

    async fn init(&mut self, window: Window) -> Result<(), RendererError> {
        cfg_if! {
            if #[cfg(feature = "sync")] {
                let size = self.game.physics.settings.window_size;
            } else {
                let size = self.game.config.window_size;
            }
        };

        self.wgpu = WgpuState(Some(Self::init_static(window, &self.instance, size).await?));
        self.shader.create(&self.wgpu)?;

        Ok(())
    }

    fn update_window_size(&mut self, size: Vec2) {
        cfg_if! {
            if #[cfg(feature = "sync")] {
                self.game.physics.settings.window_size = size;
            } else {
                self.game.config.window_size = size;
            }
        };

        self.shader.globals.resolution = size.to_array();

        self.wgpu.config.width = size.x as u32;
        self.wgpu.config.height = size.y as u32;
    }

    fn draw(&mut self) -> Result<(), DrawError> {
        let surface_tex = self.wgpu.surface.get_current_texture()?;
        let device = &self.wgpu.device;

        let msaa_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa circle texture"),
            size: surface_tex.texture.size(),
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: self.wgpu.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let msaa_view = msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let surface_view = surface_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("circle command encoder"),
        });

        self.shader.update()?;

        if self.shader.uninit() {
            self.shader.create(&self.wgpu)?;
        }

        self.wgpu.queue.write_buffer(
            &self.shader.globals_buf,
            0,
            bytemuck::cast_slice(&[self.shader.globals]),
        );

        self.wgpu.queue.write_buffer(
            &self.shader.prims_buf,
            0,
            bytemuck::cast_slice(&self.shader.prims),
        );

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(&surface_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            pass.set_pipeline(&self.shader.pipeline);
            pass.set_bind_group(0, &self.shader.bind_group, &[]);
            pass.set_index_buffer(self.shader.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(0, self.shader.vertex_buf.slice(..));

            pass.draw_indexed(0..self.shader.index_buf.size() as u32 / 2, 0, 0..1);
        }

        self.wgpu.queue.submit(Some(encoder.finish()));
        surface_tex.present();

        Ok(())
    }
}

impl ApplicationHandler for SimRenderer {
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.wgpu.uninit() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                info!("Got close request, quitting!");
                event_loop.exit()
            }
            WindowEvent::RedrawRequested => {
                self.draw().unwrap();
                self.wgpu.window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                self.update_window_size(Vec2::new(size.width as f32, size.height as f32));
            }
            WindowEvent::Occluded(true) => self.wgpu.window.request_redraw(),
            _ => {}
        }
    }

    fn resumed(&mut self, ev: &ActiveEventLoop) {
        // rust iife lol
        (|| {
            let win = ev.create_window(
                Window::default_attributes()
                    .with_active(true)
                    // .with_fullscreen(Some(Fullscreen::Borderless(None)))
                    .with_title("fluidsim"),
            )?;

            cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    // safety: SimRenderer was Box::leaked
                    // just a little bit of jank
                    // wasm async lifetimes are cursed

                    let ptr = self as *mut _;
                    let num = ptr as usize;

                    task::block_on(async move {
                        let ptr = num as *mut Self;
                        let this = unsafe { &mut *ptr };

                        this.init(win).await.unwrap();
                    })
                } else {
                    task::block_on(self.init(win))?;
                }
            }

            Ok::<_, ResumeError>(())
        })()
        .unwrap();
    }
}

// impl State {
//     /// Update the panel (mouse/keyboard) as well as sending good mouse data
//     fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
//         let propagate = !self.panel.update(ctx);

//         let mouse = &ctx.mouse;
//         let left_pressed = mouse.button_pressed(MouseButton::Left) && propagate;
//         let any_pressed = (mouse.button_pressed(MouseButton::Right) || left_pressed) && propagate;
//         let data = any_pressed.then_some(MouseState {
//             px: mouse.position().into(),
//             is_left: left_pressed,
//         });

//         if data != self.mouse {
//             self.mouse = data;
//             ipc::physics_send(ToPhysics::UpdateMouse(data));
//         }

//         Ok(())
//     }

//     fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
//         let (width, height) = ctx.gfx.drawable_size();
//         let (halfw, halfh) = (width / 2., height / 2.);

//         // create and setup the canvas
//         let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

//         // make the center at zero,zero to make my life easier
//         canvas.set_screen_coordinates(graphics::Rect::new(
//             -width / 2.0,
//             -height / 2.0,
//             width,
//             height,
//         ));

//         // grab the current scene and create a mesh
//         let sc = self.physics.get();
//         let mut mesh = graphics::MeshBuilder::new();

//         // draw to mesh from scene
//         sc.draw(&mut mesh)?;

//         // draw the mesh to the canvas
//         canvas.draw(&graphics::Mesh::from_data(ctx, mesh.build()), Vec2::ZERO);

//         // draw the panel to the canvas
//         canvas.draw(&*self.panel, DrawParam::new().dest([-halfw, -halfh]));

//         // draw the current fps
//         let (ref mut old_tps, ref mut old_count) = self.tps_data;

//         let fps = format!("Rendering FPS: {:.2}", ctx.time.fps());
//         let physics_fps = format!(
//             "Physics TPS: {}",
//             if *old_count >= 10 {
//                 *old_tps = self.physics.tps();
//                 *old_count = 0;
//                 self.physics.tps()
//             } else {
//                 *old_count += 1;
//                 *old_tps
//             }
//         );

//         let fps_text = graphics::Text::new(fps);
//         let physics_fps_text = graphics::Text::new(physics_fps);

//         let fps_dest = Vec2::new(-halfw + 10.0, halfh - 20.0);
//         let physics_fps_dest = Vec2::new(-halfw + 10.0, halfh - 40.0);

//         canvas.draw(&fps_text, fps_dest);
//         canvas.draw(&physics_fps_text, physics_fps_dest);

//         canvas.finish(ctx)?;

//         ggez::timer::yield_now();

//         Ok(())
//     }

//     fn resize_event(
//         &mut self,
//         ctx: &mut ggez::Context,
//         width: f32,
//         height: f32,
//     ) -> Result<(), ggez::GameError> {
//         let Some(wpos) = self.panel.update_wpos(ctx)? else {
//             return Ok(());
//         };

//         let wsize = Vec2::new(width, height);
//         self.panel.set_window(wsize, wpos);

//         Ok(())
//     }

//     fn key_down_event(
//         &mut self,
//         ctx: &mut ggez::Context,
//         input: KeyInput,
//         _repeated: bool,
//     ) -> Result<(), ggez::GameError> {
//         let PhysicalKey::Code(kc) = input.event.physical_key else {
//             return Ok(());
//         };

//         match kc {
//             KeyCode::Space => ipc::physics_send(ToPhysics::Pause),
//             KeyCode::ArrowRight => ipc::physics_send(ToPhysics::Step),
//             KeyCode::KeyR => ipc::physics_send(ToPhysics::Reset),
//             KeyCode::KeyC => {
//                 debug!("Toggling config panel");
//                 self.panel.toggle();
//             }
//             KeyCode::KeyH => {
//                 debug!("Toggling help text");
//                 self.panel.toggle_help();
//             }
//             KeyCode::KeyQ if input.mods.control_key() => {
//                 info!("Got ctrl+q, quitting!");
//                 ipc::physics_send(ToPhysics::Kill);
//                 ctx.request_quit();
//             }
//             _ => (),
//         }

//         Ok(())
//     }
// }
