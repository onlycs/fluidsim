use core::f32;

use spirv_std::num_traits::real::Real;

pub fn smoothing(dist: f32, radius: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }

    let volume = (f32::consts::PI * radius.powi(4)) / 6.0; // calculated by wolfram alpha
    let diff = radius - dist;
    diff.powi(2) / volume
}

pub fn viscosity_smoothing(dist: f32, radius: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }

    let volume = (f32::consts::PI * radius.powi(8)) / 4.0; // calculated by wolfram alpha
    let diff2 = radius.powi(2) - dist.powi(2);
    diff2.powi(3) / volume
}

pub fn smoothing_deriv(dist: f32, radius: f32) -> f32 {
    if dist >= radius || dist == 0.0 {
        return 0.0;
    }

    let scale = 12. / radius.powi(4) * f32::consts::PI;
    (dist - radius) * scale
}

pub fn density_to_pressure(density: f32, target: f32, multiplier: f32) -> f32 {
    let err = density - target;
    err * multiplier
}
