use egui::{Button, RichText, Slider};
use glam::Quat;

use crate::{
    prelude::*,
    renderer::{
        graphics::GraphicsContext,
        shader::{lines::LineShader, physics::PhysicsShader},
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

fn degrees(rad: &mut f32) -> impl FnMut(Option<f64>) -> f64 + '_ {
    move |v| {
        if let Some(v) = v {
            *rad = v.to_radians() as f32;
        }

        f64::from(rad.to_degrees())
    }
}

impl Panel {
    #[allow(clippy::too_many_lines)]
    pub fn update<'a>(
        &'a self,
        ctx: &'a GraphicsContext,
        state: &'a mut SimulationState,
        physics: &'a mut PhysicsShader,
        lines: &'a mut LineShader,
    ) -> impl FnMut(&mut egui::Ui) + 'a {
        |ui: &mut egui::Ui| {
            let settings = physics.lease_panel();

            let mut reset = false;
            let mut reline = false;

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

                ui.add(
                    Slider::from_get_set(20.0..=120.0, degrees(&mut state.player.fov)).text("FoV"),
                )
                .changed();

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

                ui.collapsing("Particle Count", |ui| {
                    reset |= ui
                        .add(Slider::new(&mut state.init.particles.x, 1..=32).text("Particles X"))
                        .changed();

                    reset |= ui
                        .add(Slider::new(&mut state.init.particles.y, 1..=32).text("Particles Y"))
                        .changed();

                    reset |= ui
                        .add(Slider::new(&mut state.init.particles.z, 1..=32).text("Particles Z"))
                        .changed();

                    ui.add_space(5.0);
                });

                ui.collapsing("Boundary Size", |ui| {
                    reline |= ui
                        .add(Slider::new(&mut state.init.box_size.x, 0.0..=16.0).text("Boundary X"))
                        .changed();

                    reline |= ui
                        .add(Slider::new(&mut state.init.box_size.y, 0.0..=16.0).text("Boundary Y"))
                        .changed();

                    reline |= ui
                        .add(Slider::new(&mut state.init.box_size.z, 0.0..=16.0).text("Boundary Z"))
                        .changed();

                    ui.add_space(5.0);
                });

                ui.collapsing("Boundary Rotation", |ui| {
                    let (mut y, mut x, mut z) = state.init.box_quat.to_euler(glam::EulerRot::YXZEx);

                    reline |= ui
                        .add(
                            Slider::from_get_set(-90.0..=90.0, degrees(&mut x))
                                .text("Roll (About X)"),
                        )
                        .changed();

                    reline |= ui
                        .add(
                            Slider::from_get_set(-90.0..=90.0, degrees(&mut y))
                                .text("Pitch (About Y)"),
                        )
                        .changed();

                    reline |= ui
                        .add(
                            Slider::from_get_set(-90.0..=90.0, degrees(&mut z))
                                .text("Yaw (About Z)"),
                        )
                        .changed();

                    state.init.box_quat = Quat::from_euler(glam::EulerRot::YXZEx, y, x, z);

                    ui.add_space(5.0);
                });

                reset |= ui
                    .add(Slider::new(&mut state.init.gap, 0.0..=3.0).text("Initial Gap"))
                    .changed();

                ui.add(Slider::new(&mut settings.particle_radius, 0.0..=1.0).text("Radius"))
                    .changed();

                ui.add_space(25.0);
                ui.label(RichText::new("Presets").size(TEXT_SIZE).strong());

                if ui
                    .add_sized([240., 30.], Button::new("Default Settings"))
                    .clicked()
                {
                    reset = true;
                    *settings = SimSettings {
                        box_size: settings.box_size,
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

            if reset || reline {
                physics.reset(ctx, state);
            }

            if reline {
                lines.rebuild(&ctx.device, state.init.box_size, state.init.box_quat);
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
