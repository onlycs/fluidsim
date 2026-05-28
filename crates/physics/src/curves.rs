use core::f32;

use spirv_std::num_traits::real::Real;

// \int_0^{2\pi}
//      \int_0^\pi
//          \int_0^h [(h-r)^2 \times r^2 \times \sin \theta] dr
//      d\theta
// d\phi
//
// = (\int_0^{2\pi} d\phi)
//   (\int_0^{\pi} \sin \theta d\theta)
//   (\int_0^h [(h-r)^2 \times r^2] dr)
//
// = \frac{2\pi \times h^5}{15}
pub fn density(dist: f32, radius: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }

    const VOLUME: f32 = 2.0 * f32::consts::PI / 15.0;
    const VOLUME_INVERSE: f32 = 1.0 / VOLUME;

    (radius - dist).powi(2) * (VOLUME_INVERSE / radius.powi(5))
}

// \int_0^{2\pi}
//      \int_0^\pi
//          \int_0^h [(h-r)^2 \times r^3 \times \sin \theta] dr
//      d\theta
// d\phi
//
// = (\int_0^{2\pi} d\phi)
//   (\int_0^{\pi} \sin \theta d\theta)
//   (\int_0^h [(h-r)^2 \times r^3] dr)
//
// = \frac{\pi \times h^6}{15}
pub fn density_near(dist: f32, radius: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }

    const VOLUME: f32 = f32::consts::PI / 15.0;
    const VOLUME_INVERSE: f32 = 1.0 / VOLUME;

    (radius - dist).powi(3) * (VOLUME_INVERSE / radius.powi(6))
}

// \frac{d}{dr} (h-r)^2 = -2(h-r)
// \frac{-2(h-r)}{V} = (r-h) * \frac{15}{\pi \times h^5}
pub fn density_deriv(dist: f32, radius: f32) -> f32 {
    if dist >= radius || dist == 0.0 {
        return 0.0;
    }

    const SCALE: f32 = 15.0 / f32::consts::PI;
    (dist - radius) * (SCALE / radius.powi(5))
}

// \frac{d}{dr} (h-r)^3 = -3(h-r)
// \frac{-3(h-r)}{V} = (r-h) * \frac{45}{\pi \times h^6}
pub fn ndensity_deriv(dist: f32, radius: f32) -> f32 {
    if dist >= radius || dist == 0.0 {
        return 0.0;
    }

    const SCALE: f32 = 45.0 / f32::consts::PI;
    (dist - radius).powi(2) * (SCALE / radius.powi(6))
}

// \int_0^{2\pi}
//      \int_0^\pi
//          \int_0^h [(h^2-r^2)^3 \times r^2 \times \sin \theta] dr
//      d\theta
// d\phi
//
// = (\int_0^{2\pi} d\phi)
//   (\int_0^{\pi} \sin \theta d\theta)
//   (\int_0^h [(h^2-r^2)^3 \times r^2] dr)
//
// = \frac{64 \pi \times h^9}{315}
pub fn viscosity(dist: f32, radius: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }

    const VOLUME: f32 = 64.0 * f32::consts::PI / 315.0;
    const VOLUME_INVERSE: f32 = 1.0 / VOLUME;

    (radius.powi(2) - dist.powi(2)).powi(3) * (VOLUME_INVERSE / radius.powi(9))
}

pub fn density_to_pressure(density: f32, target: f32, multiplier: f32) -> f32 {
    let err = density - target;
    err * multiplier
}
