use crate::prelude::*;

use async_std::{channel::Sender, task};
use eframe::{
    egui::{self, Pos2, Response, Vec2, ViewportBuilder},
    NativeOptions,
};
use winit::platform::wayland::EventLoopBuilderExtWayland;

pub struct ConfigPanel {
    gravity: f32,
    gravity_slider: Option<Response>,
}

impl ConfigPanel {
    pub fn new() -> Self {
        Self {
            gravity: 0.0,
            gravity_slider: None,
        }
    }
}

impl eframe::App for ConfigPanel {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Config Panel");

            ui.horizontal(|ui| {
                ui.label("Gravity");
                self.gravity_slider = Some(
                    ui.add(egui::Slider::new(&mut self.gravity, -20.0..=20.0).text("Gravity")),
                );
            });
        });

        if let Some(sl) = &self.gravity_slider
            && sl.changed()
        {
            info!("Updating gravity");
            ipc::physics_send(ToPhysics::Gravity(self.gravity));
        }
    }
}

pub fn run() {
    loop {
        eframe::run_native(
            "Fluid Simulation",
            NativeOptions {
                event_loop_builder: Some(Box::new(|h| {
                    h.with_any_thread(true);
                })),
                viewport: ViewportBuilder {
                    inner_size: Some(Vec2::new(150., 40.)),
                    position: Some(Pos2::new(0., 0.)),
                    ..Default::default()
                },
                ..Default::default()
            },
            Box::new(|_| Ok(Box::new(ConfigPanel::new()))),
        )
        .unwrap();

        info!("Killed config panel, restarting...");
    }
}
