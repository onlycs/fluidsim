use glam::{Quat, UVec3, Vec3};
use gpu_shared::{DEFAULT_BOX_SIZE, DEFAULT_PARTICLES};

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
    pub particles: UVec3,
    pub box_size: Vec3,
    pub box_quat: Quat,
    pub gap: f32,
}

impl Default for InitialConditions {
    fn default() -> Self {
        Self {
            particles: DEFAULT_PARTICLES,
            box_size: DEFAULT_BOX_SIZE,
            box_quat: Quat::IDENTITY,
            gap: 0.05,
        }
    }
}
