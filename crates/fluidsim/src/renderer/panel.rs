use crate::prelude::*;
use egui::{Button, Context, RichText, Slider};

const TEXT_SIZE: f32 = 16.0;

pub struct Panel {
    show_self: bool,
    show_help: bool,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            show_self: true,
            show_help: true,
        }
    }
}

#[derive(Debug)]
pub struct UpdateData<'a> {
    pub reset: &'a mut bool,
    pub retessellate: &'a mut bool,
}

impl Panel {
    /// Returns a function that should be used once to update the panel and synchronize updated settings
    pub fn update<'a>(
        &'a self,
        sim: &'a mut SimSettings,
        gfx: &'a mut GraphicsSettings,
        init: &'a mut InitialConditions,
        update: UpdateData<'a>,
    ) -> impl FnMut(&Context) + 'a {
        let UpdateData {
            reset,
            retessellate,
        } = update;

        |ctx: &Context| {
            if !self.show_self {
                return;
            }

            egui::Window::new("Simulation Settings").show(ctx, |ui| {
                ui.label(RichText::new("Graphics Settings").size(TEXT_SIZE).strong());

                ui.add(Slider::new(&mut gfx.speed, 0.5..=2.0).text("Speed (multiplier)"))
                    .changed();

                ui.add(Slider::new(&mut gfx.step_time, 1.0..=60.0).text("Step Time (ms)"))
                    .changed();

                ui.add(Slider::new(&mut gfx.steps_per_frame, 1..=5).text("Steps per Frame"))
                    .changed();

                ui.add_space(25.0);
                ui.label(RichText::new("Physics Settings").size(TEXT_SIZE).strong());

                ui.add(Slider::new(&mut sim.gravity, -20.0..=20.0).text("Gravity"));

                ui.add(
                    Slider::new(&mut sim.collision_damping, 0.0..=1.0).text("Collision Dampening"),
                );

                ui.add_space(25.0);
                ui.label(RichText::new("SPH Settings").size(TEXT_SIZE).strong());

                ui.add(Slider::new(&mut sim.smoothing_radius, 0.01..=4.0).text("Smoothing Radius"));

                ui.add(Slider::new(&mut sim.target_density, 0.0..=200.0).text("Target Density"));

                ui.add(
                    Slider::new(&mut sim.pressure_multiplier, 0.0..=300.0)
                        .text("Pressure Multiplier"),
                );

                ui.add(
                    Slider::new(&mut sim.viscosity_strength, 0.0..=1.0).text("Viscosity Strength"),
                );

                ui.add_space(25.0);
                ui.label(RichText::new("Mouse Settings").size(TEXT_SIZE).strong());

                ui.add(
                    Slider::new(&mut sim.interaction_radius, 0.0..=10.0).text("Interaction Radius"),
                );

                ui.add(
                    Slider::new(&mut sim.interaction_strength, 0.0..=100.0)
                        .text("Interaction Strength"),
                );

                ui.add_space(25.0);
                ui.label(RichText::new("Initial Conditions").size(TEXT_SIZE).strong());

                *reset |= ui
                    .add(
                        Slider::new(&mut init.particles.x, 1.0..=100.0)
                            .integer()
                            .text("Particles X"),
                    )
                    .changed();

                *reset |= ui
                    .add(
                        Slider::new(&mut init.particles.y, 1.0..=100.0)
                            .integer()
                            .text("Particles Y"),
                    )
                    .changed();

                *reset |= ui
                    .add(Slider::new(&mut init.gap, 0.0..=3.0).text("Initial Gap"))
                    .changed();

                *retessellate |= ui
                    .add(Slider::new(&mut sim.particle_radius, 0.0..=1.0).text("Radius"))
                    .changed();

                ui.add_space(25.0);
                ui.label(RichText::new("Presets").size(TEXT_SIZE).strong());

                if ui
                    .add_sized([180., 30.], Button::new("Default Settings"))
                    .clicked()
                {
                    *reset = true;
                    *sim = SimSettings {
                        window_size: sim.window_size,
                        ..SimSettings::default()
                    }
                }

                if ui
                    .add_sized([180., 30.], Button::new("Zero Gravity"))
                    .clicked()
                {
                    *reset = true;
                    *sim = SimSettings {
                        window_size: sim.window_size,
                        ..SimSettings::zero_gravity()
                    };
                }

                if self.show_help {
                    ui.add_space(10.0);
                    ui.label("Press space to pause/play the simulation");
                    ui.label("Press the right arrow to step the simulation");
                    ui.label("Use the left mouse button to pull particles");
                    ui.label("Use the right mouse button to push particles");
                    ui.label("Press 'R' to restart");
                    ui.label("Press 'C' to toggle this panel");
                    ui.label("Press 'H' to toggle this help text");
                }
            });

            // validation
            if init.particles.x <= 0.0 {
                init.particles.x = 1.0;
            }

            if init.particles.y <= 0.0 {
                init.particles.y = 1.0;
            }

            if sim.particle_radius <= 0.0 {
                sim.particle_radius = 0.01;
            }

            init.particles.x = init.particles.x.round();
            init.particles.y = init.particles.y.round();
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_self(&mut self) {
        self.show_self = !self.show_self;
    }
}
