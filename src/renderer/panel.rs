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
        settings: &'a mut SimSettings,
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

            egui::Window::new("Simulation Settings").show(&ctx, |ui| {
                ui.label(RichText::new("Graphics Settings").size(TEXT_SIZE).strong());

                ui.add(Slider::new(&mut settings.speed, 0.5..=2.0).text("Speed (multiplier)"))
                    .changed();

                ui.add(Slider::new(&mut settings.step_time, 1.0..=60.0).text("Step Time (ms)"))
                    .changed();

                ui.add(Slider::new(&mut settings.steps_per_frame, 1..=5).text("Steps per Frame"))
                    .changed();

                ui.add_space(25.0);
                ui.label(RichText::new("Physics Settings").size(TEXT_SIZE).strong());

                ui.add(Slider::new(&mut settings.gravity, -20.0..=20.0).text("Gravity"))
                    .changed();

                ui.add(
                    Slider::new(&mut settings.collision_dampening, 0.0..=1.0)
                        .text("Collision Dampening"),
                )
                .changed();

                ui.add_space(25.0);
                ui.label(RichText::new("SPH Settings").size(TEXT_SIZE).strong());

                ui.add(
                    Slider::new(&mut settings.smoothing_radius, 0.01..=4.0)
                        .text("Smoothing Radius"),
                )
                .changed();

                ui.add(
                    Slider::new(&mut settings.target_density, 0.0..=200.0).text("Target Density"),
                )
                .changed();

                ui.add(
                    Slider::new(&mut settings.pressure_multiplier, 0.0..=300.0)
                        .text("Pressure Multiplier"),
                )
                .changed();

                ui.add(
                    Slider::new(&mut settings.viscosity_strength, 0.0..=1.0)
                        .text("Viscosity Strength"),
                )
                .changed();

                ui.add_space(25.0);
                ui.label(RichText::new("Mouse Settings").size(TEXT_SIZE).strong());

                ui.add(
                    Slider::new(&mut settings.interaction_radius, 0.0..=10.0)
                        .text("Interaction Radius"),
                )
                .changed();

                ui.add(
                    Slider::new(&mut settings.interaction_strength, 0.0..=100.0)
                        .text("Interaction Strength"),
                )
                .changed();

                ui.add_space(25.0);
                ui.label(RichText::new("Initial Conditions").size(TEXT_SIZE).strong());

                *reset |= ui
                    .add(
                        Slider::new(&mut settings.particles.x, 1.0..=100.0)
                            .integer()
                            .text("Particles X"),
                    )
                    .changed();

                *reset |= ui
                    .add(
                        Slider::new(&mut settings.particles.y, 1.0..=100.0)
                            .integer()
                            .text("Particles Y"),
                    )
                    .changed();

                *reset |= ui
                    .add(Slider::new(&mut settings.gap, 0.0..=3.0).text("Initial Gap"))
                    .changed();

                *retessellate |= ui
                    .add(Slider::new(&mut settings.radius, 0.0..=1.0).text("Radius"))
                    .changed();

                ui.add_space(25.0);
                ui.label(RichText::new("Presets").size(TEXT_SIZE).strong());

                if ui
                    .add_sized([180., 30.], Button::new("Default Settings"))
                    .clicked()
                {
                    *reset = true;
                    *settings = SimSettings {
                        window_size: settings.window_size,
                        ..SimSettings::default()
                    }
                }

                if ui
                    .add_sized([180., 30.], Button::new("Zero Gravity"))
                    .clicked()
                {
                    *reset = true;
                    *settings = SimSettings {
                        window_size: settings.window_size,
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
            if settings.particles.x <= 0.0 {
                settings.particles.x = 1.0;
            }

            if settings.particles.y <= 0.0 {
                settings.particles.y = 1.0;
            }

            if settings.radius <= 0.0 {
                settings.radius = 0.01;
            }

            settings.particles.x = settings.particles.x.round();
            settings.particles.y = settings.particles.y.round();
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_self(&mut self) {
        self.show_self = !self.show_self;
    }
}
