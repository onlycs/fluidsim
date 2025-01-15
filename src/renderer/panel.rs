use super::*;
use crate::prelude::*;

const TEXT_SIZE: f32 = 16.0;

pub fn panel(mut ctx: EguiContexts, mut state: ResMut<GraphicsState>) {
    let mut updated = false;
    let mut reset = false;

    let res = egui::Window::new("Physics Config")
        .show(ctx.ctx_mut(), |ui| {
            ui.label(RichText::new("GFX Settings").size(TEXT_SIZE).strong());

            updated |= ui
                .add(Slider::new(&mut state.physics_cfg.fps, 50.0..=255.0).text("TPS"))
                .changed();

            ui.add_space(25.0);
            ui.label(RichText::new("Physics Settings").size(TEXT_SIZE).strong());

            updated |= ui
                .add(Slider::new(&mut state.physics_cfg.gravity, -20.0..=20.0).text("Gravity"))
                .changed();

            updated |= ui
                .add(
                    Slider::new(&mut state.physics_cfg.collision_dampening, 0.0..=1.0)
                        .text("Collision Dampening"),
                )
                .changed();

            ui.add_space(25.0);
            ui.label(RichText::new("SPH Settings").size(TEXT_SIZE).strong());

            updated |= ui
                .add(
                    Slider::new(&mut state.physics_cfg.smoothing_radius, 0.01..=4.0)
                        .text("Smoothing Radius"),
                )
                .changed();

            updated |= ui
                .add(
                    Slider::new(&mut state.physics_cfg.target_density, 0.0..=200.0)
                        .text("Target Density"),
                )
                .changed();

            updated |= ui
                .add(
                    Slider::new(&mut state.physics_cfg.pressure_multiplier, 0.0..=300.0)
                        .text("Pressure Multiplier"),
                )
                .changed();

            updated |= ui
                .add(
                    Slider::new(&mut state.physics_cfg.viscosity_strength, 0.0..=1.0)
                        .text("Viscosity Strength"),
                )
                .changed();

            ui.add_space(25.0);
            ui.label(RichText::new("Mouse Settings").size(TEXT_SIZE).strong());

            updated |= ui
                .add(
                    Slider::new(&mut state.physics_cfg.interaction_radius, 0.0..=10.0)
                        .text("Interaction Radius"),
                )
                .changed();

            updated |= ui
                .add(
                    Slider::new(&mut state.physics_cfg.interaction_strength, 0.0..=100.0)
                        .text("Interaction Strength"),
                )
                .changed();

            ui.add_space(25.0);
            ui.label(RichText::new("Initial Conditions").size(TEXT_SIZE).strong());

            reset |= ui
                .add(
                    Slider::new(&mut state.physics_cfg.particles.x, 1.0..=100.0)
                        .integer()
                        .text("Particles X"),
                )
                .changed();

            reset |= ui
                .add(
                    Slider::new(&mut state.physics_cfg.particles.y, 1.0..=100.0)
                        .integer()
                        .text("Particles Y"),
                )
                .changed();

            reset |= ui
                .add(Slider::new(&mut state.physics_cfg.gap, 0.0..=3.0).text("Initial Gap"))
                .changed();

            updated |= ui
                .add(Slider::new(&mut state.physics_cfg.radius, 0.0..=1.0).text("Radius"))
                .changed();

            ui.add_space(25.0);
            ui.label(RichText::new("Presets").size(TEXT_SIZE).strong());

            if ui
                .add_sized([180., 30.], egui::Button::new("Default Settings"))
                .clicked()
            {
                reset = true;
                state.physics_cfg = Default::default();
            }

            if ui
                .add_sized([180., 30.], egui::Button::new("Zero Gravity"))
                .clicked()
            {
                reset = true;
                state.physics_cfg = SimSettings::zero_gravity();
            }

            if state.panel_cfg.help {
                ui.add_space(10.0);
                ui.label("Press space to pause/play the simulation");
                ui.label("Press the right arrow to step the simulation");
                ui.label("Press 'C' to toggle this panel");
                ui.label("Press 'H' to toggle the help text");
            }
        })
        .unwrap()
        .response;

    if state.physics_cfg.particles.x <= 0.0 {
        state.physics_cfg.particles.x = 1.0;
    }

    if state.physics_cfg.particles.y <= 0.0 {
        state.physics_cfg.particles.y = 1.0;
    }

    if state.physics_cfg.radius <= 0.0 {
        state.physics_cfg.radius = 0.01;
    }

    state.physics_cfg.particles.x = state.physics_cfg.particles.x.round();
    state.physics_cfg.particles.y = state.physics_cfg.particles.y.round();

    if updated || reset {
        ipc::physics_send(ToPhysics::Settings(state.physics_cfg));
    }

    if reset {
        ipc::physics_send(ToPhysics::Reset);
    }

    if res.contains_pointer() != state.mouse.panel_hover {
        state.mouse.panel_hover = res.contains_pointer();
        ipc::physics_send(ToPhysics::UpdateMouse(state.mouse));
    }
}
