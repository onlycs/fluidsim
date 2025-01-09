use crate::prelude::*;
use std::ops::{Deref, DerefMut};

use super::egui_translator::EguiTranslator;
use egui::Slider;
use physics::settings::SimSettings;

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
    ) -> Result<Option<Vec2>, ggez::GameError> {
        let wpos = match ctx.gfx.window_position() {
            Ok(ppos) => Vec2::new(ppos.x as f32, ppos.y as f32),

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

                Vec2::new(posx as f32, posy as f32)
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
                .add(Slider::new(&mut self.settings.fps, 50.0..=255.0).text("TPS"))
                .changed();

            ui.add_space(25.0);

            updated |= ui
                .add(Slider::new(&mut self.settings.gravity, -20.0..=20.0).text("Gravity"))
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
                    Slider::new(&mut self.settings.smoothing_radius, 0.01..=4.0)
                        .text("Smoothing Radius"),
                )
                .changed();

            updated |= ui
                .add(
                    Slider::new(&mut self.settings.target_density, 0.0..=50.0)
                        .text("Target Density"),
                )
                .changed();

            updated |= ui
                .add(
                    Slider::new(&mut self.settings.pressure_multiplier, 0.0..=300.0)
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
                .add(Slider::new(&mut self.settings.gap, 0.0..=3.0).text("Initial Gap"))
                .changed();

            updated |= ui
                .add(Slider::new(&mut self.settings.radius, 0.0..=1.0).text("Radius"))
                .changed();

            if self.help {
                ui.add_space(10.0);
                ui.label("Press space to pause/play the simulation");
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

        if self.settings.radius <= 0.0 {
            self.settings.radius = 0.01;
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

    pub fn set_window(&mut self, size: Vec2, pos: Vec2) {
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
