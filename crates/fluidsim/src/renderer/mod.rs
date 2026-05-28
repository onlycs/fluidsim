mod buffers;
mod egui;
mod graphics;
mod input;
mod panel;
mod shader;
mod state;
mod text;

use glam::{Quat, Vec3};
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
        egui::UiRenderer,
        graphics::{GraphicsContext, GraphicsInitError},
        input::{HumanInput, InputProcessor},
        panel::Panel,
        shader::{circles::CircleShader, lines::LineShader, physics::PhysicsShader},
        state::SimulationState,
        text::PerformanceDisplay,
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

    #[snafu(display("At {location}: text display error\n{source}"))]
    PerformanceDisplay {
        source: text::TextError,
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
    circle: CircleShader,
    lines: LineShader,

    ui: UiRenderer,
    panel: Panel,

    perf: PerformanceDisplay,
    input: InputProcessor,
    state: SimulationState,
}

impl RendererInit {
    fn update_window_size(&mut self, size: UVec2) {
        let scale = self.ctx.window.scale_factor() as f32;

        // update subsystems
        self.perf.resize(size, scale);
        self.circle.resize(&self.ctx, size);

        // reconfigure surface
        self.ctx.config.width = size.x;
        self.ctx.config.height = size.y;
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

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("fluidsim/encoder"),
        });

        let framesteps = self.state.gfx.steps_per_frame;
        let dtime = self.state.dtime() / framesteps as f32;

        for _ in 0..framesteps {
            self.physics.update(queue, &mut encoder, dtime);
        }

        // draw particles
        self.circle.draw(
            &self.ctx,
            &mut encoder,
            &surface_view,
            &self.state,
            &self.physics.udata,
            &self.lines,
            self.ctx.window.inner_size().to_uvec2(),
        );

        // draw fps counter
        self.perf
            .render(&self.ctx, &mut encoder, &surface_view, &self.state)
            .context(PerformanceDisplaySnafu)?;

        // draw egui
        self.ui.draw(
            &self.ctx,
            &mut encoder,
            &surface_view,
            self.panel.update(
                &self.ctx,
                &mut self.state,
                &mut self.physics,
                &mut self.lines,
            ),
        );

        queue.submit(Some(encoder.finish()));
        surface_tex.present();

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
        let size = window.inner_size().to_uvec2();

        let panel = Panel::default();
        let ctx = GraphicsContext::new(window, size)
            .await
            .context(GraphicsInitSnafu)?;

        let mut phyiscs = PhysicsShader::new(&ctx.device, &ctx.queue);
        let vs = CircleShader::new(&ctx, phyiscs.buffers(), size);
        let ui = UiRenderer::new(&ctx);
        let perf = PerformanceDisplay::new(&ctx);
        let mut state = SimulationState::new();

        phyiscs.reset(&ctx, &mut state);

        *self = Self::Init(RendererInit {
            physics: phyiscs,
            lines: LineShader::new(
                &ctx.device,
                &ctx.config.format,
                vs.globals_buf(),
                state.init.box_size,
                state.init.box_quat,
            ),
            circle: vs,
            ctx,
            ui,
            perf,
            panel,
            input: InputProcessor::default(),
            state,
        });

        Ok(())
    }
}

const TRANSLATE_STRENGTH: f32 = 0.175;
const ROTATE_STRENGTH: f32 = 0.025;

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
            HumanInput::Keyboard { ui, motion } => {
                for key in ui {
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

                for key in motion {
                    match key {
                        KeyCode::KeyW => {
                            let q = this.state.player.q_yaw();
                            this.state.player.translate += q * Vec3::NEG_Z * TRANSLATE_STRENGTH;
                        }
                        KeyCode::KeyS => {
                            let q = this.state.player.q_yaw();
                            this.state.player.translate += q * Vec3::Z * TRANSLATE_STRENGTH;
                        }
                        KeyCode::KeyA => {
                            let q = this.state.player.q_yaw();
                            this.state.player.translate += q * Vec3::NEG_X * TRANSLATE_STRENGTH;
                        }
                        KeyCode::KeyD => {
                            let q = this.state.player.q_yaw();
                            this.state.player.translate += q * Vec3::X * TRANSLATE_STRENGTH;
                        }
                        KeyCode::ShiftLeft | KeyCode::ShiftRight => {
                            this.state.player.translate.y += TRANSLATE_STRENGTH;
                        }
                        KeyCode::ControlLeft | KeyCode::ControlRight => {
                            this.state.player.translate.y -= TRANSLATE_STRENGTH;
                        }
                        KeyCode::Numpad8 => {
                            let rot = Quat::from_rotation_x(ROTATE_STRENGTH);
                            this.state.player.q *= rot;
                        }
                        KeyCode::Numpad2 => {
                            let rot = Quat::from_rotation_x(-ROTATE_STRENGTH);
                            this.state.player.q *= rot;
                        }
                        KeyCode::Numpad4 => {
                            let rot = Quat::from_rotation_y(ROTATE_STRENGTH);
                            this.state.player.q = rot * this.state.player.q;
                        }
                        KeyCode::Numpad6 => {
                            let rot = Quat::from_rotation_y(-ROTATE_STRENGTH);
                            this.state.player.q = rot * this.state.player.q;
                        }
                        _ => {}
                    }
                }
            }
            HumanInput::Mouse { position, lmb, rmb } => {
                this.physics.set_mouse(position, lmb, rmb);
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
                this.update_window_size(UVec2::new(size.width, size.height));
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
