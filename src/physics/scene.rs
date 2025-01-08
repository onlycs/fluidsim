use core::f32;
use std::time::Instant;

use crate::prelude::*;
use ggez::graphics::{self, FillOptions};
use itertools::Itertools;
use vec2::{Acceleration2, Length2, Velocity2};

use super::settings::SimSettings;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator,
    IntoParallelRefMutIterator, ParallelBridge, ParallelIterator,
};

fn rad1((x, y): (isize, isize)) -> [(isize, isize); 9] {
    [
        (x - 1, y - 1),
        (x, y - 1),
        (x + 1, y - 1),
        (x - 1, y),
        (x, y),
        (x + 1, y),
        (x - 1, y + 1),
        (x, y + 1),
        (x + 1, y + 1),
    ]
}

#[derive(Clone, Debug, Default)]
pub struct SpacialLookup {
    pub lookup: Vec<usize>,
    pub starts: Vec<usize>,
}

impl SpacialLookup {
    fn pos_to_cell(pos: Length2, settings: SimSettings) -> (isize, isize) {
        let cell_size = settings.smoothing_radius * 2.0;
        let x = f32::from(pos.x / cell_size).floor();
        let y = f32::from(pos.y / cell_size).floor();

        (x as isize, y as isize)
    }

    fn cell_key((x, y): (isize, isize), settings: SimSettings) -> usize {
        let px = 17;
        let py = 31;
        let h =
            (x * px + y * py).rem_euclid((settings.particles.x * settings.particles.y) as isize);

        h as usize
    }

    fn update(&mut self, positions: &Vec<Length2>, settings: SimSettings) {
        let mut lookup = positions
            .par_iter()
            .map(|pos| {
                let cell = Self::pos_to_cell(*pos, settings);
                let key = Self::cell_key(cell, settings);
                key
            })
            .enumerate()
            .collect::<Vec<_>>();

        lookup.sort_by_key(|n| n.1);

        let keys: Vec<_>;

        (self.lookup, keys) = lookup.into_iter().collect();
        self.starts = vec![usize::MAX; self.lookup.len()];

        for (i, key) in keys.iter().enumerate().dedup_by(|(_, a), (_, b)| a == b) {
            self.starts[*key] = i;
        }
    }

    fn get_by_key(&self, key: usize) -> &[usize] {
        let idx = *self.starts.get(key).unwrap_or(&usize::MAX);

        if idx == usize::MAX {
            return &[];
        }

        let end = self.starts[key + 1..].iter().find(|n| n != &&usize::MAX);
        let end = *end.unwrap_or(&self.starts.len());

        if end <= idx {
            return &[];
        }

        &self.lookup[idx..end]
    }
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub positions: Vec<Length2>,
    pub densities: Vec<f32>,
    pub velocities: Vec<Velocity2>,
    pub lookup: SpacialLookup,
    pub settings: SimSettings,
}

// creation and updating scene settings, etc
impl Scene {
    pub fn new() -> Self {
        let settings = SimSettings::default();

        let mut this = Self {
            settings,
            positions: Vec::new(),
            densities: Vec::new(),
            velocities: Vec::new(),
            lookup: SpacialLookup::default(),
        };

        this.reset();

        this
    }

    pub fn absbounds(&self) -> Length2 {
        self.settings.size / 2.0
    }

    pub fn reset(&mut self) {
        let nx = self.settings.particles.x;
        let ny = self.settings.particles.y;
        let gap = self.settings.gap;
        let size = self.settings.radius;

        let bbox_size = Length2::of(nx * size + (nx - 1.0) * gap, ny * size + (ny - 1.0) * gap);
        let tl = Length2::of(
            (-bbox_size.x / 2.0) + self.settings.radius / 2.0,
            (bbox_size.y / 2.0) - self.settings.radius / 2.0,
        );

        self.positions.clear();
        self.densities.clear();
        self.velocities.clear();

        for i in 0..(nx as i32) {
            for j in 0..(ny as i32) {
                let offset = Length2::of((size + gap) * i as f32, -(size + gap) * j as f32);
                let pos = tl + offset;

                self.positions.push(pos);
            }
        }

        self.velocities = vec![Velocity2::zero(); self.positions.len()];
        self.densities = vec![0.0; self.positions.len()];
        self.lookup.update(&self.positions, self.settings);
    }

    pub fn update_settings(&mut self, settings: SimSettings) {
        self.settings = settings;
    }

    pub fn draw(&self, mesh: &mut graphics::MeshBuilder) -> Result<(), ggez::GameError> {
        for p in &self.positions {
            let Length2 { x, y } = *p;
            let xpx = x.get::<pixel>();
            let ypx = y.get::<pixel>();

            mesh.circle(
                graphics::DrawMode::Fill(FillOptions::default()),
                [xpx, ypx],
                self.settings.radius.get::<pixel>(),
                0.1,
                graphics::Color::WHITE,
            )?;
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }
}

// actual physics
impl Scene {
    pub fn apply_gravity(&mut self) {
        let at = self.settings.gravity * self.settings.tick_delay;
        self.velocities.par_iter_mut().for_each(|v| *v += at);
    }

    pub fn update_densities(&mut self) {
        (0..self.len())
            .into_par_iter()
            .map(|i| {
                Self::density(
                    &self.positions,
                    &self.lookup,
                    self.settings,
                    self.positions[i],
                )
            })
            .collect_into_vec(&mut self.densities);
    }

    pub fn update_positions(&mut self) {
        // d = vt
        self.positions
            .par_iter_mut()
            .zip(self.velocities.par_iter())
            .for_each(|(pos, vel)| *pos += *vel * self.settings.tick_delay);
    }

    pub fn collide(&mut self) {
        let Length2 { x, y } = self.absbounds();

        self.positions
            .par_iter_mut()
            .zip(self.velocities.par_iter_mut())
            .for_each(|(pos, vel)| {
                if (pos.y.abs() + self.settings.radius) > y {
                    let sign = pos.y.get::<meter>().signum();

                    pos.y = y * sign + self.settings.radius * -sign;
                    vel.y *= -self.settings.collision_dampening;
                }

                if (pos.x.abs() + self.settings.radius) > x {
                    let sign = pos.x.get::<meter>().signum();

                    pos.x = x * sign + self.settings.radius * -sign;
                    vel.x *= -self.settings.collision_dampening;
                }
            });
    }

    pub fn apply_pressure_forces(&mut self) {
        (0..self.len())
            .into_par_iter()
            .map(|i| {
                let force = Self::pressure_force(
                    &self.positions,
                    &self.densities,
                    &self.lookup,
                    self.settings,
                    i,
                );
                force / self.densities[i]
            })
            .zip(self.velocities.par_iter_mut())
            .for_each(|(accel, vel)| {
                *vel += Acceleration2::from_glam::<mps2>(accel) * self.settings.tick_delay
            });
    }

    pub fn update(&mut self) {
        self.lookup.update(&self.positions, self.settings);
        self.apply_gravity();
        self.update_densities();
        self.apply_pressure_forces();
        self.update_positions();
        self.collide();
    }
}

// density and pressure calculations
// no partial borrowing here is just absurd
// comon rust do better
impl Scene {
    /// - dist: distance between two particles
    /// - radius: smoothing radius
    fn smoothing(dist: Length, radius: Length) -> f32 {
        if dist >= radius {
            return 0.0;
        }

        let volume = (f32::consts::PI * radius.get::<cm>().powi(4)) / 6.0;
        let diff = radius.get::<cm>() - dist.get::<cm>();
        diff.powi(2) / volume
    }

    fn smoothing_deriv(dist: Length, radius: Length) -> f32 {
        if dist >= radius || dist == Length::ZERO {
            return 0.0;
        }

        let scale = 12. / radius.get::<cm>().powi(4) * f32::consts::PI;
        (dist.get::<cm>() - radius.get::<cm>()) * scale
    }

    fn density(
        positions: &Vec<Length2>,
        lookup: &SpacialLookup,
        settings: SimSettings,
        sample: Length2,
    ) -> f32 {
        let raw = rad1(SpacialLookup::pos_to_cell(sample, settings))
            .into_iter()
            .map(|n| SpacialLookup::cell_key(n, settings))
            .flat_map(|key| lookup.get_by_key(key))
            .copied()
            .map(|pidx| {
                let dist = (positions[pidx] - sample).mag();
                let influence = Self::smoothing(dist, settings.smoothing_radius);
                settings.mass.get::<kilogram>() * influence
            })
            .sum::<f32>();

        raw
    }

    fn density_to_pressure(settings: SimSettings, density: f32) -> f32 {
        let err = density - settings.target_density;
        let pressure = err * settings.pressure_multiplier;
        pressure
    }

    fn pressure_force(
        positions: &Vec<Length2>,
        densities: &Vec<f32>,
        lookup: &SpacialLookup,
        settings: SimSettings,
        particle: usize,
    ) -> GlamVec2 {
        let mass = settings.mass.get::<kilogram>();
        let pdensity = densities[particle];
        let ppressure = Self::density_to_pressure(settings, pdensity);

        rad1(SpacialLookup::pos_to_cell(positions[particle], settings))
            .into_iter()
            .map(|n| SpacialLookup::cell_key(n, settings))
            .flat_map(|key| lookup.get_by_key(key))
            .copied()
            .filter(|pidx| pidx != &particle)
            .map(|pidx| {
                let pos = positions[pidx];
                let offset = pos - positions[particle];
                let dist = offset.mag();

                let dir = if dist == Length::ZERO {
                    GlamVec2::new(rand::random::<f32>(), rand::random::<f32>()).normalize()
                } else {
                    offset / dist
                };

                let slope = Self::smoothing_deriv(dist, settings.smoothing_radius);
                let pressure = Self::density_to_pressure(settings, densities[pidx]);
                let pressure_shared = (pressure + ppressure) / 2.0;

                pressure_shared * dir * slope * mass / densities[pidx]
            })
            .sum()
    }
}
