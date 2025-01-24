use lyon::tessellation::TessellationError;
use std::{backtrace::Backtrace, panic::Location};
use thiserror::Error;
use wgpu::CreateSurfaceError;
use winit::error::{self, EventLoopError, OsError};

#[derive(Error, Debug)]
pub enum InitError {
    #[error("At {location}: winit: Event Loop Error:\n{source}")]
    EventLoop {
        #[from]
        source: EventLoopError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}

#[derive(Error, Debug)]
pub enum RendererError {
    #[error("At {location}: wgpu: Failed to create surface:\n{source}")]
    CreateSurface {
        #[from]
        source: CreateSurfaceError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("At {location}: wgpu: Failed to request device:\n{source}")]
    RequestDevice {
        #[from]
        source: wgpu::RequestDeviceError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("At {location}: lyon: Tessellation error:\n{source}")]
    Tessellation {
        #[from]
        source: TessellationError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("No texture format")]
    NoTextureFormat,

    #[error("No adapter found")]
    NoAdapter,
}

#[derive(Error, Debug)]
pub enum ResumeError {
    #[error("At {location}: renderer error:\n{source}")]
    Renderer {
        #[from]
        source: RendererError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("At {location}: winit: failed to create window:\n{source}")]
    Winit {
        #[from]
        source: OsError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}

#[derive(Error, Debug)]
pub enum DrawError {
    #[error("At {location}: wgpu: surface error:\n{source}")]
    CreatePipeline {
        #[from]
        source: wgpu::SurfaceError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("At {location}: lyon: failed to tessellate path:\n{source}")]
    Tessellate {
        #[from]
        source: TessellationError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}
