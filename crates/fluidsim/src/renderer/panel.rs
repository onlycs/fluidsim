use egui::{Button, RichText, Slider};

use crate::{
    prelude::*,
    renderer::{
        graphics::GraphicsContext,
        shader::{compute::PhysicsShader, vertex::CircleShader},
        state::SimulationState,
    },
};

const TEXT_SIZE: f32 = 16.0;

pub struct Panel {
    show: bool,
    show_help: bool,
}

impl Default for Panel {
    fn default() -> Self {
        Self {
            show: true,
            show_help: true,
        }
    }
}

impl Panel {
    #[allow(clippy::too_many_lines)]
    pub fn update<'a>(
        &'a self,
        ctx: &'a GraphicsContext,
        state: &'a mut SimulationState,
        physics: &'a mut PhysicsShader,
        circles: &'a mut CircleShader,
    ) -> impl FnMut(&mut egui::Ui) + 'a {
        |ui: &mut egui::Ui| {
            let settings = physics.lease_panel();

            let mut reset = false;
            let mut retessellate = false;

            if !self.show {
                return;
            }

            egui::Window::new("Simulation Settings").show(ui.ctx(), |ui| {
                ui.label(RichText::new("Graphics Settings").size(TEXT_SIZE).strong());

                ui.add(Slider::new(&mut state.gfx.speed, 0.5..=2.0).text("Speed (multiplier)"))
                    .changed();

                ui.add(Slider::new(&mut state.gfx.step_time, 1.0..=60.0).text("Step Time (ms)"))
                    .changed();

                ui.add(Slider::new(&mut state.gfx.steps_per_frame, 1..=5).text("Steps per Frame"))
                    .changed();

                ui.add(Slider::new(circles.lease_zoom(), 0.5..=1.0).text("Zoom"));

                ui.add_space(25.0);
                ui.label(RichText::new("Physics Settings").size(TEXT_SIZE).strong());

                ui.add(Slider::new(&mut settings.gravity, -20.0..=20.0).text("Gravity"));

                ui.add(
                    Slider::new(&mut settings.collision_damping, 0.0..=1.0)
                        .text("Collision Dampening"),
                );

                ui.add_space(25.0);
                ui.label(RichText::new("SPH Settings").size(TEXT_SIZE).strong());

                ui.add(
                    Slider::new(&mut settings.smoothing_radius, 0.01..=4.0)
                        .text("Smoothing Radius"),
                );

                reset |= ui
                    .add(
                        Slider::new(&mut settings.target_density, 0.1..=175.0)
                            .text("Target Density"),
                    )
                    .changed();

                ui.add(
                    Slider::new(&mut settings.pressure_multiplier, 1.0..=700.0)
                        .text("Pressure Multiplier"),
                );

                ui.add(
                    Slider::new(&mut settings.near_pressure_multiplier, 0.0..=50.0)
                        .text("Near Pressure Multiplier"),
                );

                ui.add(
                    Slider::new(&mut settings.viscosity_strength, 0.0..=1.0)
                        .text("Viscosity Strength"),
                );

                ui.add_space(25.0);
                ui.label(RichText::new("Mouse Settings").size(TEXT_SIZE).strong());

                ui.add(
                    Slider::new(&mut settings.interaction_radius, 0.0..=10.0)
                        .text("Interaction Radius"),
                );

                ui.add(
                    Slider::new(&mut settings.interaction_strength, 0.0..=100.0)
                        .text("Interaction Strength"),
                );

                ui.add_space(25.0);
                ui.label(RichText::new("Initial Conditions").size(TEXT_SIZE).strong());

                reset |= ui
                    .add(
                        Slider::new(&mut state.init.particles.x, 1..=100)
                            .integer()
                            .text("Particles X"),
                    )
                    .changed();

                reset |= ui
                    .add(
                        Slider::new(&mut state.init.particles.y, 1..=100)
                            .integer()
                            .text("Particles Y"),
                    )
                    .changed();

                reset |= ui
                    .add(Slider::new(&mut state.init.gap, 0.0..=3.0).text("Initial Gap"))
                    .changed();

                retessellate |= ui
                    .add(Slider::new(&mut settings.particle_radius, 0.0..=1.0).text("Radius"))
                    .changed();

                ui.add_space(25.0);
                ui.label(RichText::new("Presets").size(TEXT_SIZE).strong());

                if ui
                    .add_sized([240., 30.], Button::new("Default Settings"))
                    .clicked()
                {
                    reset = true;
                    *settings = SimSettings {
                        window_size: settings.window_size,
                        ..SimSettings::default()
                    }
                }

                if self.show_help {
                    ui.add_space(10.0);
                    ui.label("Press space to pause/play the simulation");
                    ui.label("Press the right arrow to step the simulation");
                    ui.label("Use the left mouse button to pull particles");
                    ui.label("Use the right mouse button to push particles");
                    ui.label("Press 'R' to restart");
                    ui.label("Press 'C' to toggle this panel");
                    ui.label("Press 'P' to toggle debug performance info");
                    ui.label("Press 'H' to toggle this help text");
                }
            });

            if settings.particle_radius <= 0.0 {
                settings.particle_radius = 0.01;
            }

            if retessellate {
                circles.retesselate(ctx, settings.particle_radius).unwrap();
            }

            if reset {
                physics.reset(ctx, state);
            }
        }
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_self(&mut self) {
        self.show = !self.show;
    }
}
