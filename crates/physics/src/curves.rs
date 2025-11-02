use core::f32;

use spirv_std::num_traits::real::Real;

pub fn smoothing(dist: f32, radius: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }

    const VOLUME: f32 = f32::consts::PI / 6.0; // calculated by wolfram alpha, multiply by radius^4
    const VOLUME_INVERSE: f32 = 1.0 / VOLUME; // divide by radius^4

    (radius - dist).powi(2) * (VOLUME_INVERSE / radius.powi(4))
}

pub fn smoothing_near(dist: f32, radius: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }

    const VOLUME: f32 = f32::consts::PI / 10.0; // calculated by wolfram alpha, multiply by radius^5
    const VOLUME_INVERSE: f32 = 1.0 / VOLUME; // divide by radius^5

    (radius - dist).powi(3) * (VOLUME_INVERSE / radius.powi(5))
}

pub fn smoothing_deriv(dist: f32, radius: f32) -> f32 {
    if dist >= radius || dist == 0.0 {
        return 0.0;
    }

    const VOLUME_DERIV: f32 = 12.0 / f32::consts::PI; // calculated by wolfram alpha, divide by radius^4

    (dist - radius) * (VOLUME_DERIV / radius.powi(4))
}

pub fn nsmoothing_deriv(dist: f32, radius: f32) -> f32 {
    if dist >= radius || dist == 0.0 {
        return 0.0;
    }

    const VOLUME_DERIV: f32 = 30.0 / f32::consts::PI; // calculated by wolfram alpha, divide by radius^5

    -(radius - dist).powi(2) * (VOLUME_DERIV / radius.powi(5))
}

pub fn viscosity_smoothing(dist: f32, radius: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }

    const VOLUME: f32 = f32::consts::PI / 4.0; // calculated by wolfram alpha, multiply by radius^8
    const VOLUME_INVERSE: f32 = 1.0 / VOLUME; // divide by radius^8

    (radius.powi(2) - dist.powi(2)).powi(3) * (VOLUME_INVERSE / radius.powi(8))
}

pub fn density_to_pressure(density: f32, target: f32, multiplier: f32) -> f32 {
    let err = density - target;
    err * multiplier
}
