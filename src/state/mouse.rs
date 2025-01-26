use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct MouseState {
    pub px: Vec2,
    pub left: bool,
    pub right: bool,
}

impl MouseState {
    pub fn intensity(&self) -> f32 {
        if !self.active() {
            return 0.0;
        }

        if self.left {
            1.0
        } else {
            -1.0
        }
    }

    pub fn active(&self) -> bool {
        self.left || self.right
    }
}
