use std::ops::{Deref, DerefMut};

use egui::Slider;
use physics::settings::SimSettings;

use super::egui_translator::EguiTranslator;
use crate::prelude::*;

pub struct Panel {
    egui: EguiTranslator,
    settings: SimSettings,
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
    pub fn update(&mut self, ctx: &mut ggez::Context) {
        let panel_ctx = self.egui.ctx();
        let mut updated = false;

        egui::Window::new("Simulation Settings").show(&panel_ctx, |ui| {
            updated |= ui
                .add(Slider::new(&mut self.settings.gravity, -100.0..=100.0).text("Gravity"))
                .changed();

            updated |= ui
                .add(Slider::new(&mut self.settings.tps, 0..=500).text("TPS"))
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

        if updated {
            ipc::physics_send(ToPhysics::Settings(self.settings));
        }

        self.egui.update(ctx);
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
