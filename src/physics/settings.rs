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

    pub particles: Vec2,
    pub gap: f32,
    pub radius: f32,

    pub size: Vec2,
    pub position: Vec2,
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            dtime: 1.8,
            fps: 120.0,

            gravity: 9.8,
            collision_dampening: 0.40,

            smoothing_radius: 0.60,
            target_density: 25.0,
            pressure_multiplier: 180.0,

            particles: Vec2::new(70., 70.),
            gap: 0.05,
            radius: 0.05,

            // window size and position
            size: Vec2::new(800., 600.),
            position: Vec2::ZERO,

            mass: 1.0,
        }
    }
}
