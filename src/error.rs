use ggez::GameError;
use std::{backtrace::Backtrace, panic::Location};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error("At {location}: ggez: game error:\n{source}")]
    GameError {
        #[from]
        source: GameError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}
