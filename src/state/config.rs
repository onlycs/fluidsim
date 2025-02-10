use crate::prelude::*;
use bytemuck::Zeroable;

pub type SimSettings = crate::renderer::wgsl_compute::types::Settings;

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
            dtime: 0.002,

            gravity: 9.8,
            collision_damping: 0.40,

            smoothing_radius: 0.60,
            #[cfg(not(target_arch = "wasm32"))]
            target_density: 35.0,
            #[cfg(target_arch = "wasm32")]
            target_density: 20.0,

            pressure_multiplier: 150.0,
            viscosity_strength: 0.06,

            interaction_radius: 4.0,
            interaction_strength: 90.0,

            // window size
            #[cfg(not(target_arch = "wasm32"))]
            window_size: Vec2::new(1200., 800.),
            #[cfg(target_arch = "wasm32")]
            window_size: Vec2::new(1500., 1000.),

            #[cfg(not(target_arch = "wasm32"))]
            num_particles: 80 * 80,
            #[cfg(target_arch = "wasm32")]
            num_particles: 30 * 30,

            mass: 1.0,
            particle_radius: 0.035,

            ..Self::zeroed()
        }
    }
}

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
    pub particles: Vec2,
    pub gap: f32,
}

impl Default for InitialConditions {
    fn default() -> Self {
        Self {
            #[cfg(not(target_arch = "wasm32"))]
            particles: Vec2::new(80., 80.),
            #[cfg(target_arch = "wasm32")]
            particles: Vec2::new(30., 30.),
            gap: 0.05,
        }
    }
}
