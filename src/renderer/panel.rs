use std::ops::Deref;

use egui::Slider;

use super::egui_translator::EguiTranslator;
use crate::prelude::*;

macro_rules! configurable {
    (
        pub struct PanelData {
            $($field:ident: $ty:ty,)*
        } => $checker:ident
    ) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct PanelData {
            $(
                pub $field: $ty,
            )*
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
        pub struct $checker {
            $(
                pub $field: bool,
            )*
        }
    };
}

configurable!(
    pub struct PanelData {
        gravity: f32,
    } => UpdateChecks
);

impl UpdateChecks {
    pub fn apply_all(&self, data: &PanelData) {
        if self.gravity {
            info!("Gravity changed to: {}", data.gravity);
            ipc::physics_send(ToPhysics::Gravity(data.gravity));
        }
    }
}

impl Default for PanelData {
    fn default() -> Self {
        Self { gravity: -9.8 }
    }
}

#[derive(Default)]
pub struct Panel {
    egui: EguiTranslator,
    data: PanelData,
}

impl Panel {
    pub fn update(&mut self, ctx: &mut ggez::Context) {
        let panel_ctx = self.egui.ctx();
        let mut checks = UpdateChecks::default();

        egui::Window::new("Config Panel").show(&panel_ctx, |ui| {
            checks.gravity = ui
                .add(Slider::new(&mut self.data.gravity, -20.0..=20.0).text("Gravity"))
                .changed();
        });

        // borrowing panel as mut
        drop(panel_ctx);

        checks.apply_all(&self.data);
        self.egui.update(ctx);
    }
}

impl Deref for Panel {
    type Target = EguiTranslator;

    fn deref(&self) -> &Self::Target {
        &self.egui
    }
}
