use bytemuck::{Pod, Zeroable};

use crate::prelude::*;

pub type MouseState = crate::renderer::wgsl_compute::types::MouseState;

#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct RawMouseState {
    position: [f32; 2],
    clickmask: u32,
    _pad: u32,
}

impl MouseState {
    pub fn new(px: Vec2, left: bool, right: bool) -> Self {
        Self {
            position: px,
            clickmask: (left as u32) | ((right as u32) << 1),
            ..Self::zeroed()
        }
    }

    pub fn intensity(&self) -> f32 {
        if !self.active() {
            return 0.0;
        }

        if self.left() {
            1.0
        } else {
            -1.0
        }
    }

    pub fn active(&self) -> bool {
        self.left() || self.right()
    }

    pub fn left(&self) -> bool {
        self.clickmask & 1 != 0
    }

    pub fn right(&self) -> bool {
        self.clickmask & 2 != 0
    }

    pub fn update(&mut self, px: Option<Vec2>, left: bool, right: bool) {
        self.position = px.unwrap_or(self.position);
        self.clickmask = (left as u32) | ((right as u32) << 1);
    }

    pub fn to_raw(&self) -> RawMouseState {
        RawMouseState {
            position: [self.position.x, self.position.y],
            clickmask: self.clickmask,
            _pad: 0,
        }
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self { ..Self::zeroed() }
    }
}
