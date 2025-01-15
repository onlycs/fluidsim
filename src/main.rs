#![feature(
    error_generic_member_access,
    never_type,
    let_chains,
    if_let_guard,
    trait_alias,
    trivial_bounds,
    iter_collect_into
)]

extern crate async_std;
extern crate bevy;
extern crate bevy_egui;
extern crate bytemuck;
extern crate egui;
extern crate futures;
extern crate ggez;
extern crate glam;
extern crate itertools;
extern crate lazy_static;
extern crate log;
extern crate rand;
extern crate rayon;
extern crate thiserror;

pub mod error;
pub mod gradient;
pub mod ipc;
pub mod physics;
pub mod prelude;
pub mod renderer;
pub mod shared;

use bevy::window::{WindowMode, WindowResolution};
use bevy_egui::EguiPlugin;
use error::InitError;
use prelude::*;

fn main() -> Result<(), InitError> {
    info!("Starting up");

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Index(0)),
                resolution: WindowResolution::default().with_scale_factor_override(1.0),
                ..Default::default()
            }),
            ..Default::default()
        }),
        EguiPlugin,
    ));

    app.insert_resource(ClearColor(Color::BLACK));

    app.add_systems(Startup, renderer::resources);
    app.add_systems(Startup, renderer::setup.after(renderer::resources));
    // app.add_systems(Update, renderer::mouse);
    app.add_systems(Update, renderer::panel::panel);
    app.add_systems(Update, renderer::draw);

    app.run();

    Ok(())
}
