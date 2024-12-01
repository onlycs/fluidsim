use std::{backtrace::Backtrace, panic::Location};

use thiserror::Error;
use winit::error::EventLoopError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("At {location}: Event loop error:\n{source}")]
    EventLoopError {
        #[from]
        source: EventLoopError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}
