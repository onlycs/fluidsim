mod buffers;
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
        compute::PhysicsShader,
        egui::UiRenderer,
        graphics::{GraphicsContext, GraphicsInitError},
        input::{HumanInput, InputProcessor},
        panel::Panel,
        performance::PerformanceDisplay,
        shader::{
            compute,
            vertex::{CircleShader, VertexShaderError},
        },
        state::SimulationState,
    },
};

#[derive(Debug, Snafu)]
pub(crate) enum DrawError {
    #[snafu(display("At {location}: Surface lost during draw"))]
    SurfaceLost {
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("At {location}: wgpu: poll error\n{source}"))]
    Poll {
        source: wgpu::PollError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("At {location}: vertex shader update error\n{source}"))]
    VertexShaderUpdate {
        source: VertexShaderError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("At {location}: performance display error\n{source}"))]
    PerformanceDisplay {
        source: performance::TextError,
        #[snafu(implicit)]
        location: Location,
    },
}

#[derive(Debug, Snafu)]
pub(crate) enum RendererInitError {
    #[snafu(display("At {location}: graphics init error\n{source}"))]
    GraphicsInit {
        source: GraphicsInitError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("At {location}: vertex shader init error\n{source}"))]
    VertexShaderInit {
        source: VertexShaderError,
        #[snafu(implicit)]
        location: Location,
    },
}

#[derive(Debug, Snafu)]
pub(crate) enum ResumeError {
    #[snafu(display("At {location}: winit: window create error\n{source}"))]
    WindowCreate {
        source: winit::error::OsError,
        #[snafu(implicit)]
        location: Location,
    },

    #[snafu(display("At {location}: renderer init error\n{source}"))]
    RendererInit {
        source: RendererInitError,
        #[snafu(implicit)]
        location: Location,
    },
}

pub(crate) struct RendererInit {
    ctx: GraphicsContext,
    physics: PhysicsShader,
    vertex: CircleShader,

    ui: UiRenderer,
    panel: Panel,

    perf: PerformanceDisplay,
    input: InputProcessor,
    state: SimulationState,
}

impl RendererInit {
    fn update_window_size(&mut self, size: Vec2) {
        let scale = self.ctx.window.scale_factor() as f32;

        // update subsystems
        self.physics.resize(size, &self.ctx, &mut self.state);
        self.perf.resize(size, scale);
        self.vertex.globals.resolution = size;

        // reconfigure surface
        self.ctx.config.width = size.x as u32;
        self.ctx.config.height = size.y as u32;
        self.ctx.reconfigure_surface();
    }

    #[allow(clippy::too_many_lines)]
    fn draw(&mut self) -> Result<(), DrawError> {
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;

        let surface_tex = match self.ctx.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(tex) => tex,
            CurrentSurfaceTexture::Suboptimal(tex) => {
                self.ctx.reconfigure_surface();
                tex
            }
            CurrentSurfaceTexture::Outdated => {
                self.ctx.reconfigure_surface();
                return Ok(());
            }
            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Validation => return Ok(()),
            CurrentSurfaceTexture::Lost => return SurfaceLostSnafu.fail(),
        };

        let surface_view = surface_tex
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let msaa_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("circle/texture:msaa"),
            size: surface_tex.texture.size(),
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: self.ctx.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let msaa_view = msaa_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluidsim/encoder"),
        });

        let framesteps = self.state.gfx.steps_per_frame;
        let dtime = self.state.dtime() / framesteps as f32;
        let mut readback_buffers = vec![];

        for _ in 0..framesteps {
            readback_buffers.push(self.physics.update(device, queue, &mut encoder, dtime));
        }

        self.vertex
            .update(&self.ctx, self.physics.udata.particle_radius())
            .context(VertexShaderUpdateSnafu)?;

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
                0..self.physics.udata.num_particles(),
            );
        }

        // draw fps counter
        self.perf
            .render(&self.ctx, &mut encoder, &surface_view)
            .context(PerformanceDisplaySnafu)?;

        // draw egui
        {
            self.ui.draw(
                &self.ctx,
                &mut encoder,
                &surface_view,
                self.panel.update(
                    &self.ctx,
                    &mut self.state,
                    &mut self.physics,
                    &mut self.vertex,
                ),
            );
        }

        queue.submit(Some(encoder.finish()));
        surface_tex.present();

        for buf in readback_buffers {
            let p = Arc::clone(&self.perf.data);
            self.physics.pipelines.profile(queue, buf, move |profile| {
                *p.lock().unwrap() = profile;
            });
        }

        device.poll(wgpu::PollType::Poll).context(PollSnafu)?;

        Ok(())
    }
}

pub(crate) enum Renderer {
    Uninit,
    Init(RendererInit),
}

impl Renderer {
    pub(crate) fn new() -> Self {
        Self::Uninit
    }

    async fn init(&mut self, window: Window) -> Result<(), RendererInitError> {
        let size = window.inner_size().to_vec2();

        let panel = Panel::default();
        let gfx = GraphicsContext::new(window, size)
            .await
            .context(GraphicsInitSnafu)?;

        let cs = PhysicsShader::new(&gfx.device, &gfx.queue);
        let vs = CircleShader::new(&gfx, cs.prims()).context(VertexShaderInitSnafu)?;
        let egui = UiRenderer::new(&gfx);
        let perf = PerformanceDisplay::new(&gfx);

        *self = Self::Init(RendererInit {
            ctx: gfx,
            physics: cs,
            vertex: vs,
            ui: egui,
            perf,
            panel,
            input: InputProcessor::default(),
            state: SimulationState::new(),
        });

        Ok(())
    }
}

impl ApplicationHandler for Renderer {
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Self::Init(this) = self else {
            return;
        };

        if this.ctx.window.id() != id {
            return;
        }

        if this.ui.event(&this.ctx.window, &event).consumed {
            return;
        }

        match this.input.process(&event) {
            HumanInput::Keyboard => {
                for key in this.input.keydown() {
                    match key {
                        KeyCode::Escape => {
                            info!("Got escape, quitting!");
                            event_loop.exit();
                        }
                        KeyCode::Space => this.state.time.toggle(),
                        KeyCode::ArrowRight => this.state.time.step(),
                        KeyCode::KeyR => this.physics.reset(&this.ctx, &mut this.state),
                        KeyCode::KeyC => this.panel.toggle_self(),
                        KeyCode::KeyH => this.panel.toggle_help(),
                        KeyCode::KeyP => this.perf.toggle(),
                        _ => {}
                    }
                }
            }
            HumanInput::Mouse => {
                this.input.write_mouse(&mut this.physics);
            }
            HumanInput::None => {}
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
                this.ctx.window.request_redraw();
                this.perf.update();
            }
            WindowEvent::Resized(size) => {
                this.update_window_size(Vec2::new(size.width as f32, size.height as f32));
                this.ctx.window.request_redraw();
            }
            WindowEvent::Occluded(false) => this.ctx.window.request_redraw(),
            _ => {}
        }
    }

    fn resumed(&mut self, ev: &ActiveEventLoop) {
        // rust iife lol
        (|| {
            let win = ev
                .create_window(Window::default_attributes().with_title("fluidsim"))
                .context(WindowCreateSnafu)?;
            win.set_fullscreen(Some(Fullscreen::Borderless(None)));
            pollster::block_on(self.init(win)).context(RendererInitSnafu)?;

            Ok::<_, ResumeError>(())
        })()
        .unwrap();
    }
}
