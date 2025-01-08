use vec2::Acceleration2;

use crate::{prelude::*, vec2::Length2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimSettings {
    pub tick_delay: Time,

    pub gravity: Acceleration2,
    pub collision_dampening: f32,

    pub smoothing_radius: Length,
    pub target_density: f32,
    pub pressure_multiplier: f32,
    pub mass: Mass,

    pub particles: GlamVec2,
    pub gap: Length,
    pub radius: Length,

    pub size: Length2,
    pub position: Length2,
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            gravity: Acceleration2::new::<mps2>(0., 0.),
            tick_delay: Time::new::<ms>(6.0),
            particles: GlamVec2::new(50., 50.),
            gap: Length::new::<cm>(0.15),
            radius: Length::new::<cm>(0.05),
            collision_dampening: 0.8,
            size: Length2::new::<pixel>(800., 600.),
            position: Length2::zero(),
            smoothing_radius: Length::new::<cm>(2.0),
            target_density: 1.0,
            pressure_multiplier: 0.1,
            mass: Mass::new::<kilogram>(1.0),
        }
    }
}
