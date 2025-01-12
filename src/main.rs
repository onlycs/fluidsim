#![feature(
    error_generic_member_access,
    never_type,
    let_chains,
    if_let_guard,
    trait_alias,
    trivial_bounds,
    stmt_expr_attributes
)]

#[macro_use]
extern crate log;
#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;
extern crate futures;
#[cfg(not(target_arch = "wasm32"))]
extern crate ggez;
#[cfg(target_arch = "wasm32")]
extern crate ggez_wasm as ggez;
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
    conf::{WindowMode, WindowSetup},
    event, ContextBuilder,
};
use renderer::State;

#[cfg(not(target_arch = "wasm32"))]
use ggez::conf::NumSamples;

fn main() -> Result<(), InitError> {
    #[cfg(not(target_arch = "wasm32"))]
    logger::init();

    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    info!("Starting up");

    // give us a window
    let (ctx, event_loop) = ContextBuilder::new("fluidsim", "angad")
        .window_setup({
            #[cfg(not(target_arch = "wasm32"))]
            {
                WindowSetup::default()
                    .title("Fluid Simulation")
                    .vsync(true)
                    .samples(NumSamples::Four)
            }

            #[cfg(target_arch = "wasm32")]
            {
                WindowSetup::default().title("Fluid Simulation")
            }
        })
        .window_mode({
            #[cfg(not(target_arch = "wasm32"))]
            {
                WindowMode::default()
                    .resizable(true)
                    .dimensions(1600., 1000.)
            }

            #[cfg(target_arch = "wasm32")]
            {
                WindowMode::default().dimensions(2048., 2048.)
            }
        })
        .build()?;

    // run our window
    #[cfg(not(target_arch = "wasm32"))]
    event::run(ctx, event_loop, State::new())?;

    #[cfg(target_arch = "wasm32")]
    event::run(ctx, event_loop, State::new());

    #[cfg(not(target_arch = "wasm32"))]
    Ok(())
}
