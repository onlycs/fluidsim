pub use core::f32;
pub use std::{sync::Arc, time::Instant};

pub use bytemuck::{Pod, Zeroable};
pub use glam::Vec2;
pub use gpu_shared::{ARRAY_LEN, MouseState, Settings as SimSettings};
pub use snafu::{Location, prelude::*};
use winit::dpi::PhysicalSize;

pub(crate) use crate::config::*;

pub trait ToVec2 {
    fn to_vec2(&self) -> Vec2;
}

impl ToVec2 for PhysicalSize<u32> {
    fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }
}
