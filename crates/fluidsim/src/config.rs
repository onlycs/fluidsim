use glam::{UVec2, uvec2};

use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphicsSettings {
    pub speed: f32,
    pub step_time: f32,
    pub steps_per_frame: u32,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            speed: 1.6,
            step_time: 6.0,
            steps_per_frame: 3,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InitialConditions {
    pub particles: UVec2,
    pub gap: f32,
}

impl Default for InitialConditions {
    fn default() -> Self {
        Self {
            particles: uvec2(60, 60),
            gap: 0.05,
        }
    }
}
