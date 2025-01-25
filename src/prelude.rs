pub(crate) use crate::{
    error::{self, *},
    ipc::{
        self,
        shared::{self, *},
        ToPhysics,
    },
    physics,
};

pub use std::sync::Arc;

pub use async_std::task;

pub use glam::Vec2;
use winit::dpi::PhysicalSize;

pub const PX_PER_UNIT: f32 = 100.0;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;

#[cfg(target_arch = "wasm32")]
pub use web_time::Instant;

pub const MIN_WINDOW: PhysicalSize<u32> = PhysicalSize::new(1200, 800);
