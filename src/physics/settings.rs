use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimSettings {
    pub dtime: f32,
    pub fps: f32,

    pub gravity: f32,
    pub collision_dampening: f32,

    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub mass: f32,

    pub interaction_radius: f32,
    pub interaction_strength: f32,

    pub particles: Vec2,
    pub gap: f32,
    pub radius: f32,

    pub size: Vec2,
    pub position: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MouseState {
    pub px: Vec2,
    pub is_left: bool,
}

impl MouseState {
    pub fn force_factor(&self) -> f32 {
        if self.is_left {
            1.0
        } else {
            -1.0
        }
    }

    pub fn intensity_factor(&self) -> f32 {
        if self.is_left {
            1.0
        } else {
            0.85
        }
    }
}

impl SimSettings {
    pub fn low_density() -> Self {
        Self {
            target_density: 25.0,
            pressure_multiplier: 190.0,
            ..Default::default()
        }
    }
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            dtime: 1.8,
            fps: 120.0,

            gravity: 9.8,
            collision_dampening: 0.40,

            smoothing_radius: 0.60,
            target_density: 40.0,
            pressure_multiplier: 150.0,

            particles: Vec2::new(80., 80.),
            gap: 0.1,
            radius: 0.025,

            interaction_radius: 5.0,
            interaction_strength: 50.0,

            // window size and position
            size: Vec2::new(800., 600.),
            position: Vec2::ZERO,

            mass: 1.0,
        }
    }
}
