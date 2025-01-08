pub(crate) use crate::ipc::{self, ToPhysics};
pub(crate) use crate::{physics, vec2};

pub use uom::ConstZero;

pub use async_std::{
    channel::{Receiver, Sender},
    task,
};

pub use ggez::glam::Vec2 as GlamVec2;
pub use uom::si::f32::{Acceleration, Force, Length, Mass, Time, Velocity};
pub use uom::si::{
    acceleration::meter_per_second_squared as mps2,
    force::newton,
    length::{centimeter as cm, meter},
    mass::{gram, kilogram},
    time::{millisecond as ms, second},
    velocity::meter_per_second as mps,
};

// define pixel as a unit
pub mod px {
    pub mod length {
        unit! {
            system: uom::si;
            quantity: uom::si::length;
            @pixel: 1.0 / (188.0 * 39.3701); "px", "pixel", "pixels";
        }
    }

    pub mod velocity {
        unit! {
            system: uom::si;
            quantity: uom::si::velocity;
            @pixel_per_second: 1.0 / (188.0 * 39.3701); "px/s", "pixel per second", "pixels per second";
        }
    }
}

pub use px::{length::pixel, velocity::pixel_per_second as pxps};
