use std::ops::{Deref, DerefMut};

use egui::Slider;
use physics::settings::SimSettings;

use super::egui_translator::EguiTranslator;
use crate::prelude::*;

#[derive(Default)]
pub struct Panel {
    egui: EguiTranslator,
    settings: SimSettings,
}

impl Panel {
    pub fn update(&mut self, ctx: &mut ggez::Context) {
        let panel_ctx = self.egui.ctx();
        let mut updated = false;

        egui::Window::new("Config Panel").show(&panel_ctx, |ui| {
            updated |= ui
                .add(Slider::new(&mut self.settings.gravity, -20.0..=20.0).text("Gravity"))
                .changed();
        });

        // borrowing panel as mut
        drop(panel_ctx);

        if updated {
            ipc::physics_send(ToPhysics::Settings(self.settings));
        }

        self.egui.update(ctx);
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
