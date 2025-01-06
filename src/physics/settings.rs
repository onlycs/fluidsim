use vec2::Acceleration2;

use crate::{prelude::*, vec2::Length2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimSettings {
    pub tick_delay: Time,

    pub gravity: Acceleration2,
    pub collision_dampening: f32,

    pub particles: GlamVec2,
    pub gap: Length,
    pub radius: Length,

    pub size: Length2,
    pub position: Length2,
}

impl Default for SimSettings {
    fn default() -> Self {
        Self {
            gravity: Acceleration2::new::<mps2>(0., 9.8),
            tick_delay: Time::new::<ms>(2.85),
            particles: GlamVec2::new(30., 30.),
            gap: Length::new::<cm>(0.3),
            radius: Length::new::<cm>(0.05),
            collision_dampening: 0.8,
            size: Length2::new::<pixel>(800., 600.),
            position: Length2::zero(),
        }
    }
}
