use core::f32;

use crate::{gradient::LinearGradient, prelude::*};
use ggez::graphics::{self, FillOptions};
use itertools::Itertools;
use physics::settings::MouseState;

use super::settings::SimSettings;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator,
    IntoParallelRefMutIterator, ParallelIterator,
};

const SCALE: f32 = 100.0;

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
    fn pos_to_cell(pos: Vec2, settings: SimSettings) -> (isize, isize) {
        let cell_size = settings.smoothing_radius;
        let x = f32::from(pos.x / cell_size).floor();
        let y = f32::from(pos.y / cell_size).floor();

        (x as isize, y as isize)
    }

    fn cell_key((x, y): (isize, isize), settings: SimSettings) -> usize {
        let px = 17;
        let py = 31;
        let num_particles = settings.particles.x * settings.particles.y;
        let h = (x * px + y * py).rem_euclid(num_particles as isize);

        h as usize
    }

    fn update(&mut self, positions: &Vec<Vec2>, settings: SimSettings) {
        let mut lookup = positions
            .par_iter()
            .map(|pos| {
                let cell = Self::pos_to_cell(*pos, settings);
                let key = Self::cell_key(cell, settings);
                key
            })
            .enumerate()
            .collect::<Vec<_>>();

        lookup.sort_by_key(|(_idx, key)| *key);

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
    pub positions: Vec<Vec2>,
    pub predictions: Vec<Vec2>,
    pub densities: Vec<f32>,
    pub velocities: Vec<Vec2>,
    pub mouse: Option<MouseState>,
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
            predictions: Vec::new(),
            lookup: SpacialLookup::default(),
            mouse: None,
        };

        this.reset();

        this
    }

    pub fn absbounds(&self) -> Vec2 {
        self.settings.size / SCALE / 2.0
    }

    /// organize the particles in a centered grid
    pub fn reset(&mut self) {
        let nx = self.settings.particles.x as usize;
        let ny = self.settings.particles.y as usize;
        let size = self.settings.radius * 2.0;
        let gap = self.settings.gap;

        let topleft = -0.5
            * Vec2::new(
                (size * nx as f32) + (gap * (nx - 1) as f32) - self.settings.radius,
                (size * ny as f32) + (gap * (ny - 1) as f32) - self.settings.radius,
            );

        self.positions.clear();
        self.densities.clear();
        self.velocities.clear();

        for i in 0..nx {
            for j in 0..ny {
                let offset = Vec2::new(
                    size * i as f32 + gap * i as f32,
                    size * j as f32 + gap * j as f32,
                );

                let urandom = Vec2::new(
                    (0.5 - rand::random::<f32>()) / 10.,
                    (0.5 - rand::random::<f32>()) / 10.,
                );

                let pos = topleft + offset + urandom;

                self.positions.push(pos);
            }
        }

        self.velocities = vec![Vec2::ZERO; self.positions.len()];
        self.densities = vec![0.0; self.positions.len()];
        self.lookup.update(&self.positions, self.settings);
    }

    // update the settings of the simulation
    pub fn update_settings(&mut self, settings: SimSettings) {
        self.settings = settings;
    }

    // draw the particles
    pub fn draw(&self, mesh: &mut graphics::MeshBuilder) -> Result<(), ggez::GameError> {
        let g = LinearGradient::new(vec![
            // #1747A2 rgb(23, 71, 162)
            (0.062, graphics::Color::from_rgb(23, 71, 162)),
            // #51FC93 rgb(81, 252, 147)
            (0.48, graphics::Color::from_rgb(81, 252, 147)),
            // #FCED06, rgb(252, 237, 6)
            (0.65, graphics::Color::from_rgb(252, 237, 6)),
            // #EF4A0C, rgb(239, 74, 12)
            (1.0, graphics::Color::from_rgb(239, 74, 12)),
        ]);

        let max_vel = 15.0;

        for (i, p) in self.positions.iter().enumerate() {
            let Vec2 { x, y } = *p;
            let xpx = x * SCALE;
            let ypx = y * SCALE;
            let vel = self.velocities[i].distance(Vec2::ZERO);
            let rel = vel / max_vel;
            let col = g.sample(rel.min(1.0));

            mesh.circle(
                graphics::DrawMode::Fill(FillOptions::default()),
                [xpx, ypx],
                self.settings.radius * SCALE,
                0.1,
                col,
            )?;
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }
}

// global updates
impl Scene {
    pub fn apply_gravity(&mut self) {
        let at = self.settings.gravity * self.settings.dtime;
        self.velocities.par_iter_mut().for_each(|v| v.y += at);
    }

    // predict with constant lookahead factor (1/120 sec) to make this consistent across fps variations
    pub fn update_predictions(&mut self) {
        (0..self.len())
            .into_par_iter()
            .map(|i| self.positions[i] + self.velocities[i] * (1. / 120.))
            .collect_into_vec(&mut self.predictions);
    }

    pub fn update_densities(&mut self) {
        (0..self.len())
            .into_par_iter()
            .map(|i| {
                Self::density(
                    &self.predictions,
                    &self.lookup,
                    self.settings,
                    self.predictions[i],
                )
            })
            .collect_into_vec(&mut self.densities);
    }

    pub fn update_positions(&mut self) {
        // d = vt
        self.positions
            .par_iter_mut()
            .zip(self.velocities.par_iter())
            .for_each(|(pos, vel)| *pos += *vel * self.settings.dtime);
    }

    // wall collision
    pub fn collide(&mut self) {
        let Vec2 { x, y } = self.absbounds();

        self.positions
            .par_iter_mut()
            .zip(self.velocities.par_iter_mut())
            .for_each(|(pos, vel)| {
                if (pos.y.abs() + self.settings.radius) > y {
                    let sign = pos.y.signum();

                    pos.y = y * sign + self.settings.radius * -sign;
                    vel.y *= -self.settings.collision_dampening;
                }

                if (pos.x.abs() + self.settings.radius) > x {
                    let sign = pos.x.signum();

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
                    &self.predictions,
                    &self.densities,
                    &self.lookup,
                    self.settings,
                    i,
                );
                force / self.densities[i]
            })
            .zip(self.velocities.par_iter_mut())
            .for_each(|(accel, vel)| *vel += accel * self.settings.dtime);
    }

    pub fn apply_interaction_force(&mut self) {
        let Some(mouse) = self.mouse else {
            return;
        };

        self.positions
            .iter()
            .zip(self.velocities.iter_mut())
            .map(|(p, v)| (Self::interaction_force(mouse, *p, *v, self.settings), v))
            .map(|(f, v)| (v, f / self.settings.mass))
            .for_each(|(v, a)| *v += a * self.settings.dtime);
    }

    pub fn update(&mut self) {
        self.apply_gravity();
        self.apply_interaction_force();
        self.update_predictions();
        self.lookup.update(&self.predictions, self.settings);
        self.update_densities();
        self.apply_pressure_forces();
        self.update_positions();
        self.collide();
    }
}

// single particle calculations
// no partial borrowing here is just absurd
// comon rust do better
impl Scene {
    /// - dist: distance between two particles
    /// - radius: smoothing radius
    fn smoothing(dist: f32, radius: f32) -> f32 {
        if dist >= radius {
            return 0.0;
        }

        let volume = (f32::consts::PI * radius.powi(4)) / 6.0; // calculated by wolfram alpha
        let diff = radius - dist;
        diff.powi(2) / volume
    }

    /// the derivitative of the smoothing function
    fn smoothing_deriv(dist: f32, radius: f32) -> f32 {
        if dist >= radius || dist == 0.0 {
            return 0.0;
        }

        let scale = 12. / radius.powi(4) * f32::consts::PI;
        (dist - radius) * scale
    }

    /// calculate the density of the scene at a given point
    fn density(
        positions: &Vec<Vec2>,
        lookup: &SpacialLookup,
        settings: SimSettings,
        sample: Vec2,
    ) -> f32 {
        let raw = rad1(SpacialLookup::pos_to_cell(sample, settings)) // for every point in the smoothing radius
            .into_iter()
            .map(|n| SpacialLookup::cell_key(n, settings))
            .flat_map(|key| lookup.get_by_key(key))
            .copied()
            .map(|pidx| {
                let dist = (positions[pidx] - sample).distance(Vec2::ZERO); // get the distance between the two points
                let influence = Self::smoothing(dist, settings.smoothing_radius);
                settings.mass * influence // smoothed density (divide by volume in smoothing fn)
            })
            .sum::<f32>();

        raw
    }

    /// pressure = density error * pressure multiplier
    fn density_to_pressure(settings: SimSettings, density: f32) -> f32 {
        let err = density - settings.target_density;
        let pressure = err * settings.pressure_multiplier;
        pressure
    }

    /// calculate the repellent force
    fn pressure_force(
        positions: &Vec<Vec2>,
        densities: &Vec<f32>,
        lookup: &SpacialLookup,
        settings: SimSettings,
        particle: usize,
    ) -> Vec2 {
        let mass = settings.mass;
        let pdensity = densities[particle];
        let ppressure = Self::density_to_pressure(settings, pdensity);

        rad1(SpacialLookup::pos_to_cell(positions[particle], settings)) // for every point in the smoothing radius
            .into_iter()
            .map(|n| SpacialLookup::cell_key(n, settings))
            .flat_map(|key| lookup.get_by_key(key))
            .copied()
            .filter(|pidx| pidx != &particle)
            .map(|pidx| {
                let pos = positions[pidx];
                let offset = pos - positions[particle];
                let dist = offset.distance(Vec2::ZERO);

                let dir = if dist == 0. {
                    Vec2::new(rand::random::<f32>(), rand::random::<f32>()).normalize()
                } else {
                    offset / dist
                };

                let slope = Self::smoothing_deriv(dist, settings.smoothing_radius); // calculate the slope of the density
                let pressure = Self::density_to_pressure(settings, densities[pidx]); // calculate the pressure of that point
                let pressure_shared = (pressure + ppressure) / 2.0; // newton's third law

                pressure_shared * dir * slope * mass / densities[pidx] // calculate the force
            })
            .sum()
    }

    // thanks mr lague i would have no idea
    fn interaction_force(
        mouse: MouseState,
        position: Vec2,
        velocity: Vec2,
        settings: SimSettings,
    ) -> Vec2 {
        let mpos = (mouse.px / SCALE) - (settings.size / SCALE / 2.0);
        let mut force = Vec2::ZERO;
        let diff = mpos - position;
        let dist2 = diff.dot(diff);

        if dist2 <= settings.interaction_radius.powi(2) {
            let dist = dist2.sqrt();
            let dir = if dist <= f32::EPSILON {
                Vec2::ZERO
            } else {
                diff / dist
            };

            let center_t = 1. - dist / settings.interaction_radius;
            force += (dir * settings.interaction_strength * mouse.intensity_factor() - velocity)
                * center_t;
        }

        force * mouse.force_factor()
    }
}
