pub(crate) use crate::{config::*, error::*, renderer};
pub use gpu_shared::{MouseState, Settings as SimSettings};

pub use std::sync::Arc;

pub use glam::Vec2;
use winit::dpi::PhysicalSize;

pub const PX_PER_UNIT: f32 = 100.0;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;

#[cfg(target_arch = "wasm32")]
pub use web_time::Instant;

#[cfg(target_arch = "wasm32")]
pub const WASM_WINDOW: PhysicalSize<u32> = PhysicalSize::new(1300, 700);

pub trait ToVec2 {
    fn to_vec2(&self) -> Vec2;
}

impl ToVec2 for PhysicalSize<u32> {
    fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}
