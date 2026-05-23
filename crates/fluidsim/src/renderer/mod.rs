mod egui;
mod graphics;
mod input;
mod panel;
mod performance;
mod shader;
mod state;

use wgpu::CurrentSurfaceTexture;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    keyboard::KeyCode,
    window::{Fullscreen, Window, WindowId},
};

use crate::{
    prelude::*,
    renderer::{
        compute::ComputeShaderContext,
        egui::EguiContext,
        graphics::GraphicsContext,
        input::{InputHelper, InputResponse},
        panel::{Panel, UpdateData},
        performance::PerformanceDisplay,
        shader::{compute, vertex::VertexShaderContext},
        state::GameState,
    },
};

pub struct RendererInit {
    gfx: GraphicsContext,
    compute: ComputeShaderContext,
    vertex: VertexShaderContext,

    egui: EguiContext,
    panel: Panel,

    perf: PerformanceDisplay,
    input: InputHelper,
    game: GameState,
}

impl RendererInit {
    fn update_window_size(&mut self, size: Vec2) {
        let scale = self.gfx.window.scale_factor() as f32;

        // apply window size
        self.compute.user.settings.window_size = size;
        self.vertex.globals.resolution = size;
        self.gfx.config.width = size.x as u32;
        self.gfx.config.height = size.y as u32;

        // reconfigure surface
        self.gfx
            .surface
            .configure(&self.gfx.device, &self.gfx.config);

        // reconfigure fps counter
        let PerformanceDisplay {
            buffer,
            font_system,
            ..
        } = &mut self.perf;

        buffer.set_size(font_system, Some(size.x * scale), Some(size.y * scale));
    }

    #[allow(clippy::too_many_lines)]
    fn draw(&mut self) -> Result<(), DrawError> {
        let device = &self.gfx.device;
        let queue = &self.gfx.queue;

        let surface_tex = match self.gfx.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(tex) => tex,
            CurrentSurfaceTexture::Suboptimal(tex) => {
                self.gfx
                    .surface
                    .configure(&self.gfx.device, &self.gfx.config);
                tex
            }
            _ => panic!(),
        };

        let surface_view = surface_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let msaa_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("msaa circle texture"),
            size: surface_tex.texture.size(),
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: self.gfx.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let msaa_view = msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("physics/encoder"),
        });

        let framesteps = self.game.gfx.steps_per_frame;
        let dtime = self.game.dtime() / framesteps as f32;
        let conditions = &self.game.init;
        let mut readback_buffers = vec![];

        for _ in 0..framesteps {
            readback_buffers.push(self.compute.update(
                device,
                queue,
                conditions,
                &mut encoder,
                dtime,
            ));
        }

        self.vertex.update(&self.gfx, self.compute.user.settings)?;

        queue.write_buffer(
            &self.vertex.globals_buf,
            0,
            bytemuck::cast_slice(&[self.vertex.globals]),
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
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            pass.set_pipeline(&self.vertex.pipeline);
            pass.set_bind_group(0, &self.vertex.bind_group, &[]);
            pass.set_index_buffer(self.vertex.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(0, self.vertex.vertex_buf.slice(..));

            pass.draw_indexed(
                0..self.vertex.index_buf.size() as u32 / 2,
                0,
                0..self.compute.user.settings.num_particles,
            );
        }

        // draw fps counter
        self.perf.render(&self.gfx, &mut encoder, &surface_view)?;

        // draw egui
        {
            let ComputeShaderContext {
                update: compute::UpdateState { reset, .. },
                user: compute::UserData { settings, .. },
                ..
            } = &mut self.compute;

            let retessellate = &mut self.vertex.retessellate;

            self.egui.draw(
                &self.gfx,
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

        for buf in readback_buffers {
            let Some(buf) = buf else { continue };
            let p = Arc::clone(&self.perf.perf);
            self.compute.pipelines.profile(queue, buf, move |profile| {
                *p.lock().unwrap() = profile;
            });
        }

        device.poll(wgpu::PollType::Poll).unwrap();

        Ok(())
    }
}

pub enum Renderer {
    Uninit,
    Init(RendererInit),
}

impl Renderer {
    pub fn new() -> Self {
        Self::Uninit
    }

    async fn init(&mut self, window: Window) -> Result<(), RendererError> {
        let size = window.inner_size().to_vec2();

        let panel = Panel::default();

        let gfx = GraphicsContext::new(window, size).await?;
        let compute = ComputeShaderContext::new(&gfx.device, &gfx.queue);
        let vertex = VertexShaderContext::new(&gfx, &compute.prims_buf())?;
        let egui = EguiContext::new(&gfx);
        let perf = PerformanceDisplay::new(&gfx, panel.show_perf());

        *self = Self::Init(RendererInit {
            gfx,
            compute,
            vertex,
            egui,
            perf,
            panel,
            input: InputHelper::default(),
            game: GameState::new(),
        });

        Ok(())
    }
}

impl ApplicationHandler for Renderer {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Self::Init(this) = self else {
            return;
        };

        if this.gfx.window.id() != id {
            return;
        }

        if this.egui.event(&this.gfx.window, &event).consumed {
            return;
        }

        match this.input.process(&event) {
            InputResponse::Keyboard => {
                for key in this.input.keydown() {
                    match key {
                        KeyCode::Escape => {
                            info!("Got escape, quitting!");
                            event_loop.exit();
                        }
                        KeyCode::Space => this.game.time.play_pause(),
                        KeyCode::ArrowRight => this.game.time.step(),
                        KeyCode::KeyR => this.compute.update.reset = true,
                        KeyCode::KeyC => this.panel.toggle_self(),
                        KeyCode::KeyH => this.panel.toggle_help(),
                        KeyCode::KeyP => this.panel.toggle_perf(),
                        _ => {}
                    }
                }
            }
            InputResponse::Mouse => {
                let (x, y) = this.input.mouse_pos;

                let state = MouseState::new([x, y].into(), this.input.lmb, this.input.rmb);
                this.compute.user.mouse = state;
                this.compute.update.mouse = true;
            }
            InputResponse::None => {}
        }

        match &event {
            WindowEvent::CloseRequested => {
                info!("Got close request, quitting!");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Err(e) = this.draw() {
                    error!("Error during draw: {e}");
                    return;
                }
                this.gfx.window.request_redraw();
                this.perf.update();
            }
            WindowEvent::Resized(size) => {
                this.update_window_size(Vec2::new(size.width as f32, size.height as f32));
                this.gfx.window.request_redraw();
            }
            WindowEvent::Occluded(false) => this.gfx.window.request_redraw(),
            _ => {}
        }
    }

    fn resumed(&mut self, ev: &ActiveEventLoop) {
        // rust iife lol
        (|| {
            let win = ev.create_window(Window::default_attributes().with_title("fluidsim"))?;
            win.set_fullscreen(Some(Fullscreen::Borderless(None)));
            pollster::block_on(self.init(win))?;

            Ok::<_, ResumeError>(())
        })()
        .unwrap();
    }
}
