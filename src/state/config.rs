use crate::prelude::*;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct RawSimSettings {
    dtime: f32,

    gravity: f32,
    collision_damping: f32,

    smoothing_radius: f32,
    target_density: f32,
    pressure_multiplier: f32,
    mass: f32,

    interaction_radius: f32,
    interaction_strength: f32,

    viscosity_strength: f32,

    num_particles: u32,
    particle_radius: f32,

    window_size: [f32; 2],
    _pad: [u32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct SimSettings {
    pub dtime: f32,

    pub gravity: f32,
    pub collision_damping: f32,

    pub smoothing_radius: f32,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub mass: f32,

    pub interaction_radius: f32,
    pub interaction_strength: f32,

    pub viscosity_strength: f32,

    pub num_particles: u32,
    pub particle_radius: f32,

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

    pub fn to_raw(&self) -> RawSimSettings {
        RawSimSettings {
            dtime: self.dtime,

            gravity: self.gravity,
            collision_damping: self.collision_damping,

            smoothing_radius: self.smoothing_radius,
            target_density: self.target_density,
            pressure_multiplier: self.pressure_multiplier,
            mass: self.mass,

            interaction_radius: self.interaction_radius,
            interaction_strength: self.interaction_strength,

            viscosity_strength: self.viscosity_strength,

            num_particles: self.num_particles,
            particle_radius: self.particle_radius,

            window_size: [self.window_size.x, self.window_size.y],
            _pad: [0; 2],
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
