#![feature(error_generic_member_access)]

extern crate ggez;
#[macro_use]
extern crate log;
extern crate skuld;

mod error;
mod logger;
mod prelude;

use error::InitError;
use ggez::ContextBuilder;

fn main() -> Result<(), InitError> {
    logger::init();
    info!("Starting up");

    let cb = ContextBuilder::new("fluidsim", "angad");
    let (mut ctx, event_loop) = cb.build()?;

    Ok(())
}
