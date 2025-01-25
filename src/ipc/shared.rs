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

    pub viscosity_strength: f32,

    pub particles: Vec2,
    pub gap: f32,
    pub radius: f32,

    pub window_size: Vec2,
}

impl SimSettings {
    pub fn zero_gravity() -> Self {
        Self {
            gravity: 0.0,
            target_density: 4.0,
            pressure_multiplier: 20.0,
            viscosity_strength: 0.5,
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
            target_density: 35.0,
            pressure_multiplier: 150.0,
            viscosity_strength: 0.06,

            particles: Vec2::new(80., 80.),
            gap: 0.05,
            radius: 0.035,

            interaction_radius: 4.0,
            interaction_strength: 90.0,

            // window size
            #[cfg(not(target_arch = "wasm32"))]
            window_size: Vec2::new(1200., 800.),
            #[cfg(target_arch = "wasm32")]
            window_size: Vec2::new(1500., 1000.),

            mass: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct MouseState {
    pub px: Vec2,
    pub left: bool,
    pub right: bool,
    pub panel_hover: bool,
}

impl MouseState {
    pub fn intensity(&self) -> f32 {
        if self.left {
            1.0
        } else {
            -1.0
        }
    }
}
