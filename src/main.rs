#![feature(
    error_generic_member_access,
    never_type,
    let_chains,
    if_let_guard,
    trait_alias,
    trivial_bounds
)]

#[macro_use]
extern crate log;
extern crate futures;
extern crate ggez;
extern crate skuld;

mod error;
mod gradient;
mod ipc;
mod logger;
mod physics;
mod prelude;
mod renderer;

use error::InitError;
use ggez::{
    conf::{NumSamples, WindowMode, WindowSetup},
    event, ContextBuilder,
};
use renderer::State;

fn main() -> Result<(), InitError> {
    logger::init();
    info!("Starting up");

    // give us a window
    let (ctx, event_loop) = ContextBuilder::new("fluidsim", "angad")
        .window_setup(
            WindowSetup::default()
                .title("Fluid Simulation")
                .vsync(true)
                .samples(NumSamples::Four),
        )
        .window_mode(
            WindowMode::default()
                .resizable(true)
                .dimensions(1600., 1000.),
        )
        .build()?;

    // run our window
    event::run(ctx, event_loop, State::new())?;

    Ok(())
}
