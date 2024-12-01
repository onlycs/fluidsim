use std::backtrace::Backtrace;
use std::panic::Location;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NewStateError {
    #[error("At {location}: wgpu: failed to create surface:\n{source}")]
    CreateSurface {
        #[from]
        source: wgpu::CreateSurfaceError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("wgpu: no adapter")]
    NoAdapter,

    #[error("wgpu: failed to request device:\n{source}")]
    RequestDevice {
        #[from]
        source: wgpu::RequestDeviceError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}
