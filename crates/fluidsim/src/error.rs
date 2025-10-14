use core::fmt;
use lyon::tessellation::TessellationError;
use std::{backtrace::Backtrace, io, panic::Location};
use thiserror::Error;
use wgpu::{CreateSurfaceError, RequestAdapterError};
use winit::error::{EventLoopError, OsError};

#[derive(Error)]
pub enum InitError {
    #[error("At {location}: winit: Event Loop Error:\n{source}")]
    EventLoop {
        #[from]
        source: EventLoopError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}

impl fmt::Debug for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n{self}")
    }
}

#[derive(Error)]
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

    #[error("No valid texture formats found. Available formats:\n{available}")]
    NoTextureFormat { available: String },

    #[error("No adapter found")]
    NoAdapter {
        #[from]
        source: RequestAdapterError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("At {location}: IO error:\n{source}")]
    Io {
        #[from]
        source: io::Error,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}

impl fmt::Debug for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n{self}")
    }
}

#[derive(Error)]
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

impl fmt::Debug for ResumeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n{self}")
    }
}

#[derive(Error)]
pub enum DrawError {
    #[error("At {location}: wgpu: surface error:\n{source}")]
    Surface {
        #[from]
        source: wgpu::SurfaceError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("At {location}: lyon: failed to tessellate:\n{source}")]
    Tessellate {
        #[from]
        source: TessellationError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("At {location}: text error:\n{source}")]
    Text {
        #[from]
        source: TextError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}

impl fmt::Debug for DrawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n{self}")
    }
}

#[derive(Error)]
pub enum TextError {
    #[error("At {location}: glyphon: failed to prepare renderer:\n{source}")]
    PrepareRenderer {
        #[from]
        source: glyphon::PrepareError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },

    #[error("At {location}: glyphon: failed to render:\n{source}")]
    Render {
        #[from]
        source: glyphon::RenderError,
        location: &'static Location<'static>,
        backtrace: Backtrace,
    },
}

impl fmt::Debug for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n{self}")
    }
}
