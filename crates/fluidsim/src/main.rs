#![warn(clippy::pedantic)]
#![allow(
    clippy::similar_names,
    clippy::missing_errors_doc,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::result_large_err,
    clippy::wildcard_imports,
    clippy::large_enum_variant,
    clippy::many_single_char_names
)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use]
extern crate tracing;

mod config;
mod logger;
mod prelude;
mod renderer;

use renderer::Renderer;
use winit::{error::EventLoopError, event_loop::EventLoop};

fn main() -> Result<(), EventLoopError> {
    logger::init();

    info!("Starting up");
    let event_loop = EventLoop::builder().build()?;
    let app = Box::leak(Box::new(Renderer::new()));

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    event_loop.run_app(app)?;

    Ok(())
}
