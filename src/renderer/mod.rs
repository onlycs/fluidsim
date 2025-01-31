use crate::prelude::*;

use egui::EguiState;
use fps::{FpsCounter, FpsState};
use input::{InputHelper, InputResponse};
use panel::{Panel, UpdateData};
use shader::{
    compute::{self, ComputeState},
    vertex::VsState,
};
use state::GameState;
use std::ops::{Deref, DerefMut};
use wgpu::PowerPreference;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::{Window, WindowId},
};

use self::compute::ComputeData;

mod egui;
mod fps;
mod input;
mod panel;
mod shader;
mod state;

pub struct WgpuData {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    window: Arc<Window>,
    config: wgpu::SurfaceConfiguration,
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

pub struct SimRenderer {
    instance: wgpu::Instance,
    wgpu: WgpuState,
    shader: VsState,
    compute: ComputeState,

    egui: EguiState,
    panel: Panel,

    fps: FpsState,
    input: InputHelper,
    game: GameState,
}

impl SimRenderer {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::from_env_or_default());

        Self {
            instance,
            wgpu: WgpuState::default(),
            game: GameState::new(),
            fps: FpsState::default(),
            shader: VsState::default(),
            compute: ComputeState::default(),
            egui: EguiState::default(),
            panel: Panel::default(),
            input: InputHelper::default(),
        }
    }

    fn uninit(&self) -> bool {
        self.wgpu.uninit()
            || self.egui.uninit()
            || self.shader.uninit()
            || self.fps.uninit()
            || self.compute.uninit()
    }

    async fn init(&mut self, window: Window) -> Result<(), RendererError> {
        let size = window.inner_size().to_vec2();

        // initialize late-init stuff
        self.wgpu.init(window, &self.instance, size).await?;
        self.compute.init(&self.wgpu);
        self.shader.init(&self.wgpu, self.compute.prims_buf())?;
        self.egui.init(&self.wgpu);
        self.fps.init(&self.wgpu)?;

        #[cfg(not(target_arch = "wasm32"))]
        self.wgpu
            .window
            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));

        #[cfg(target_arch = "wasm32")]
        self.wgpu.window.set_min_inner_size(Some(WASM_WINDOW));

        Ok(())
    }

    fn update_window_size(&mut self, size: Vec2) {
        let scale = self.wgpu.window.scale_factor() as f32;

        // apply window size
        self.compute.user.settings.window_size = size;
        self.shader.globals.resolution = size.to_array();
        self.wgpu.config.width = size.x as u32;
        self.wgpu.config.height = size.y as u32;

        // reconfigure surface
        self.wgpu
            .surface
            .configure(&self.wgpu.device, &self.wgpu.config);

        // reconfigure fps counter
        let FpsCounter {
            buffer,
            font_system,
            ..
        } = &mut *self.fps;

        buffer.set_size(
            font_system,
            Some(size.x as f32 * scale),
            Some(size.y as f32 * scale),
        );
    }

    fn draw(&mut self) -> Result<(), DrawError> {
        let device = &self.wgpu.device;

        let surface_tex = self.wgpu.surface.get_current_texture()?;

        let surface_view = surface_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

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

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("circle command encoder"),
        });

        if self.game.can_update() || self.compute.update.reset {
            let dtime = self.game.dtime();

            self.compute
                .update(&self.wgpu.queue, &self.game.init, &mut encoder, dtime);
        }

        self.shader.update(&self.wgpu, self.compute.user.settings)?;

        self.wgpu.queue.write_buffer(
            &self.shader.globals_buf,
            0,
            bytemuck::cast_slice(&[self.shader.globals]),
        );

        // draw dots
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &msaa_view,
                    resolve_target: Some(&surface_view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
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

            pass.draw_indexed(
                0..self.shader.index_buf.size() as u32 / 2,
                0,
                0..self.compute.user.settings.num_particles,
            );
        }

        // draw fps counter
        {
            self.fps.render(&self.wgpu, &mut encoder, &surface_view)?;
        }

        // draw egui
        {
            let ComputeData {
                update: compute::UpdateState { reset, .. },
                user: compute::UserData { settings, .. },
                ..
            } = &mut *self.compute;

            let retessellate = &mut self.shader.retessellate;

            self.egui.draw(
                &self.wgpu,
                &mut encoder,
                &surface_view,
                self.panel.update(
                    settings,
                    &mut self.game.gfx,
                    &mut self.game.init,
                    UpdateData {
                        reset,
                        retessellate,
                    },
                ),
            );
        }

        self.wgpu.queue.submit(Some(encoder.finish()));
        self.wgpu.device.poll(wgpu::Maintain::Wait);
        surface_tex.present();

        Ok(())
    }
}

impl ApplicationHandler for SimRenderer {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if self.uninit() {
            return;
        }

        if self.egui.event(&self.wgpu.window, &event).consumed {
            return;
        }

        match self.input.process(&event) {
            InputResponse::Keyboard => {
                for key in self.input.keydown() {
                    match key {
                        KeyCode::Escape => {
                            info!("Got escape, quitting!");
                            event_loop.exit();
                        }
                        KeyCode::Space => self.game.time.play_pause(),
                        KeyCode::ArrowRight => self.game.time.step(),
                        KeyCode::KeyR => self.compute.update.reset = true,
                        KeyCode::KeyC => self.panel.toggle_self(),
                        KeyCode::KeyH => self.panel.toggle_help(),
                        _ => {}
                    }
                }
            }
            InputResponse::Mouse => {
                let (x, y) = self.input.mouse_pos;

                let state = MouseState::new([x, y].into(), self.input.lmb, self.input.rmb);
                self.compute.user.mouse = state;
                self.compute.update.mouse = true;
            }
            _ => {}
        }

        match &event {
            WindowEvent::CloseRequested => {
                info!("Got close request, quitting!");
                event_loop.exit()
            }
            WindowEvent::RedrawRequested => {
                self.draw().unwrap();
                self.wgpu.window.request_redraw();
                self.fps.update();
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

                    wasm_bindgen_futures::spawn_local(async move {
                        let ptr = num as *mut Self;
                        let this = unsafe { &mut *ptr };

                        this.init(win).await.unwrap();
                    })
                } else {
                    pollster::block_on(self.init(win))?;
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
