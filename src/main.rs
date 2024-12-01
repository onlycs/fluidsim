#![feature(error_generic_member_access)]

extern crate log;
extern crate skuld;
extern crate wgpu;
extern crate winit;

mod app;
mod error;
mod logger;
mod prelude;

use crate::prelude::*;
use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

result!(error::AppError);

fn main() -> Result<()> {
    logger::init();

    let event_loop = EventLoop::new()?;
    let mut app = App::default();

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app)?;

    Ok(())
}