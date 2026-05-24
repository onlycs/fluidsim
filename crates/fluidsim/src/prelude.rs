pub use core::f32;
pub use std::{sync::Arc, time::Instant};

pub use bytemuck::{Pod, Zeroable};
pub use glam::{UVec2, Vec2};
pub use gpu_shared::{ARRAY_LEN, MouseState, Settings as SimSettings};
pub use snafu::{Location, prelude::*};
use winit::dpi::PhysicalSize;

pub(crate) use crate::config::*;

pub trait PhysicalSizeExt {
    fn to_uvec2(&self) -> UVec2;
}

impl PhysicalSizeExt for PhysicalSize<u32> {
    fn to_uvec2(&self) -> UVec2 {
        UVec2::new(self.width, self.height)
    }
}
