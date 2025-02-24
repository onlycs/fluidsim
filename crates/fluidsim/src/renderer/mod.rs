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
use wgpu_state::WgpuState;
use winit::{
    application::ApplicationHandler,
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
mod wgpu_state;

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
        self.shader.globals.resolution = size;
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

        buffer.set_size(font_system, Some(size.x * scale), Some(size.y * scale));
    }

    fn draw(&mut self) -> Result<(), DrawError> {
        let device = &self.wgpu.device;
        let queue = &self.wgpu.queue;

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

        let framesteps = self.game.gfx.steps_per_frame;
        let dtime = self.game.dtime().min(1. / 60.) / framesteps as f32;
        let conditions = &self.game.init;

        for _ in 0..framesteps {
            self.compute.update(queue, conditions, &mut encoder, dtime);
        }

        self.shader.update(&self.wgpu, self.compute.user.settings)?;

        queue.write_buffer(
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

            pass.draw_indexed(
                0..self.shader.index_buf.size() as u32 / 2,
                0,
                0..self.compute.user.settings.num_particles,
            );
        }

        // draw fps counter
        self.fps.render(&self.wgpu, &mut encoder, &surface_view)?;

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

        queue.submit(Some(encoder.finish()));
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
