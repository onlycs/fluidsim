use crate::prelude::*;
use num::{NumCast, ToPrimitive};
use std::ops::{Deref, DerefMut};
use vec2::Length2;

use super::egui_translator::EguiTranslator;
use egui::Slider;
use physics::settings::SimSettings;

fn uom_slider_fn<'a, T: Copy + 'static, K>(
    it: &'a mut T,
    new: fn(f32) -> T,
    get: fn(&T) -> f32,
) -> impl FnMut(Option<K>) -> K + 'a
where
    K: ToPrimitive + NumCast,
{
    return move |f| {
        if let Some(f) = f {
            *it = new(<f32 as NumCast>::from(f).unwrap());
        }

        <K as NumCast>::from(get(it)).unwrap()
    };
}

pub struct Panel {
    egui: EguiTranslator,
    pub(super) settings: SimSettings,
    help: bool,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            egui: EguiTranslator::default(),
            settings: SimSettings::default(),
            help: true,
        }
    }
}

impl Panel {
    pub fn update_wpos(
        &mut self,
        ctx: &mut ggez::Context,
    ) -> Result<Option<Length2>, ggez::GameError> {
        let wpos = match ctx.gfx.window_position() {
            Ok(ppos) => Length2::new::<pixel>(ppos.x as f32, ppos.y as f32),

            #[cfg(target_os = "linux")]
            Err(_) => {
                use hyprland::{data::Clients, shared::HyprData};

                // we could possibly have hyprland which doesn't play nice
                let clients = Clients::get().unwrap();
                let clients = clients.iter().collect::<Vec<_>>();

                let Some(client) = clients.iter().find(|client| client.class == "fluidsim") else {
                    return Ok(None);
                };

                let (posy, posx) = client.at;

                Length2::new::<pixel>(posx as f32, posy as f32)
            }

            #[cfg(not(target_os = "linux"))]
            Err(err) => return Err(err),
        };

        Ok(Some(wpos))
    }

    pub fn update(&mut self, ctx: &mut ggez::Context) {
        let Ok(Some(wpos)) = self.update_wpos(ctx) else {
            return;
        };

        self.set_window(self.settings.size, wpos);

        let panel_ctx = self.egui.ctx();
        let mut updated = false;
        let mut reset = false;

        egui::Window::new("Simulation Settings").show(&panel_ctx, |ui| {
            updated |= ui
                .add(
                    Slider::from_get_set(
                        0.5..=10.0,
                        uom_slider_fn(
                            &mut self.settings.tick_delay,
                            Time::new::<ms>,
                            Time::get::<ms>,
                        ),
                    )
                    .text("Tick Delay (ms)"),
                )
                .changed();

            ui.add_space(25.0);

            updated |= ui
                .add(
                    Slider::from_get_set(
                        -20.0..=20.0,
                        uom_slider_fn(
                            &mut self.settings.gravity.y,
                            Acceleration::new::<mps2>,
                            Acceleration::get::<mps2>,
                        ),
                    )
                    .text("Gravity (m/s/s)"),
                )
                .changed();

            updated |= ui
                .add(
                    Slider::new(&mut self.settings.collision_dampening, 0.0..=1.0)
                        .text("Collision Dampening"),
                )
                .changed();

            ui.add_space(25.0);

            updated |= ui
                .add(
                    Slider::from_get_set(
                        0.01..=4.0,
                        uom_slider_fn(
                            &mut self.settings.smoothing_radius,
                            Length::new::<cm>,
                            Length::get::<cm>,
                        ),
                    )
                    .text("Smoothing Radius (cm)"),
                )
                .changed();

            updated |= ui
                .add(
                    Slider::new(&mut self.settings.target_density, 0.0..=100.0)
                        .text("Target Density"),
                )
                .changed();

            updated |= ui
                .add(
                    Slider::new(&mut self.settings.pressure_multiplier, 0.0..=100.0)
                        .text("Pressure Multiplier"),
                )
                .changed();

            ui.add_space(25.0);

            reset |= ui
                .add(
                    Slider::new(&mut self.settings.particles.x, 1.0..=100.0)
                        .integer()
                        .text("Particles X"),
                )
                .changed();

            reset |= ui
                .add(
                    Slider::new(&mut self.settings.particles.y, 1.0..=100.0)
                        .integer()
                        .text("Particles Y"),
                )
                .changed();

            reset |= ui
                .add(
                    Slider::from_get_set(
                        0.0..=3.0,
                        uom_slider_fn(&mut self.settings.gap, Length::new::<cm>, Length::get::<cm>),
                    )
                    .text("Initial Gap (cm)"),
                )
                .changed();

            updated |= ui
                .add(
                    Slider::from_get_set(
                        0.01..=2.0,
                        uom_slider_fn(
                            &mut self.settings.radius,
                            Length::new::<cm>,
                            Length::get::<cm>,
                        ),
                    )
                    .text("Particle Radius (cm)"),
                )
                .changed();

            if self.help {
                ui.add_space(10.0);
                ui.label("Press space to pause the simulation");
                ui.label("Press the right arrow to step the simulation");
                ui.label("Press 'C' to toggle this panel");
                ui.label("Press 'H' to toggle the help text");
            }
        });

        // borrowing panel as mut
        drop(panel_ctx);

        // validation
        if self.settings.particles.x <= 0.0 {
            self.settings.particles.x = 1.0;
        }

        if self.settings.particles.y <= 0.0 {
            self.settings.particles.y = 1.0;
        }

        if self.settings.radius.get::<cm>() <= 0.0 {
            self.settings.radius = Length::new::<cm>(0.01);
        }

        self.settings.particles.x = self.settings.particles.x.round();
        self.settings.particles.y = self.settings.particles.y.round();

        if updated || reset {
            ipc::physics_send(ToPhysics::Settings(self.settings));
        }

        if reset {
            ipc::physics_send(ToPhysics::Reset);
        }

        self.egui.update(ctx);
    }

    pub fn set_window(&mut self, size: Length2, pos: Length2) {
        self.settings.size = size;
        self.settings.position = pos;

        ipc::physics_send(ToPhysics::Settings(self.settings));
    }

    pub fn toggle_help(&mut self) {
        self.help = !self.help
    }
}

impl Deref for Panel {
    type Target = EguiTranslator;

    fn deref(&self) -> &Self::Target {
        &self.egui
    }
}

impl DerefMut for Panel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.egui
    }
}
