use core::f32;

use crate::{gradient::LinearGradient, prelude::*};
use ggez::graphics::{self, FillOptions, StrokeOptions};
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

/// Break the scene into a grid for faster lookups
#[derive(Clone, Debug, Default)]
pub struct SpatialLookup {
    pub lookup: Vec<usize>,
    pub starts: Vec<usize>,
}

impl SpatialLookup {
    /// convert a position to a cell coordinate based on the search radius
    fn pos_to_cell(pos: Vec2, settings: SimSettings) -> (isize, isize) {
        let cell_size = settings.smoothing_radius;
        let x = f32::from(pos.x / cell_size).floor();
        let y = f32::from(pos.y / cell_size).floor();

        (x as isize, y as isize)
    }

    /// "hash" function for a coordinate
    fn cell_key((x, y): (isize, isize), settings: SimSettings) -> usize {
        // multiply x and y by two primes and add them together. mod by the number of particles so
        // we can keep the arrays the same length (compute shader friendly)
        let px = 17;
        let py = 31;
        let num_particles = settings.particles.x * settings.particles.y;
        let h = (x * px + y * py).rem_euclid(num_particles as isize);

        h as usize
    }

    /// update the lookup table based on positions
    fn update(&mut self, positions: &Vec<Vec2>, settings: SimSettings) {
        // get the key-index pair for a position
        let mut lookup = positions
            .par_iter()
            .map(|pos| {
                let cell = Self::pos_to_cell(*pos, settings);
                let key = Self::cell_key(cell, settings);
                key
            })
            .enumerate()
            .collect::<Vec<_>>();

        // sort by the key (this is how we're looking it up)
        lookup.sort_by_key(|(_idx, key)| *key);

        let keys: Vec<_>;
        (self.lookup, keys) = lookup.into_iter().collect();
        self.starts = vec![usize::MAX; self.lookup.len()];

        // store the starting value of every possible key (usize::MAX is the invalid value)
        for (i, key) in keys.iter().enumerate().dedup_by(|(_, a), (_, b)| a == b) {
            self.starts[*key] = i;
        }
    }

    /// get the indices of all of the particles in a cell
    fn get_by_key(&self, key: usize) -> &[usize] {
        // get the starting index of the key
        let idx = *self.starts.get(key).unwrap_or(&usize::MAX);

        // if the index is invalid, return an empty slice
        if idx == usize::MAX {
            return &[];
        }

        // find the next valid index
        let end = self.starts[key + 1..].iter().find(|n| n != &&usize::MAX);
        let end = *end.unwrap_or(&self.starts.len());

        // if the end is less than the index, return an empty slice
        if end <= idx {
            return &[];
        }

        // return the slice of the lookup table
        &self.lookup[idx..end]
    }
}

/// The main scene struct
#[derive(Clone, Debug)]
pub struct Scene {
    pub positions: Vec<Vec2>,
    pub predictions: Vec<Vec2>,
    pub densities: Vec<f32>,
    pub velocities: Vec<Vec2>,
    pub mouse: Option<MouseState>,
    pub lookup: SpatialLookup,
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
            lookup: SpatialLookup::default(),
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

        // calculate the position of the top-left particle
        let topleft = -0.5
            * Vec2::new(
                (size * nx as f32) + (gap * (nx - 1) as f32) - self.settings.radius,
                (size * ny as f32) + (gap * (ny - 1) as f32) - self.settings.radius,
            );

        // clear the arrays (reset)
        self.positions.clear();
        self.densities.clear();
        self.velocities.clear();
        self.predictions.clear();

        // create the particles
        for i in 0..nx {
            for j in 0..ny {
                let offset = Vec2::new(
                    size * i as f32 + gap * i as f32,
                    size * j as f32 + gap * j as f32,
                );

                // add a small random offset to the position because this engine is very deterministic
                let urandom = Vec2::new(
                    (0.5 - rand::random::<f32>()) / 10.,
                    (0.5 - rand::random::<f32>()) / 10.,
                );

                let pos = topleft + offset + urandom;

                self.positions.push(pos);
            }
        }

        // set the velocities and densities to the correct length
        self.velocities = vec![Vec2::ZERO; self.positions.len()];
        self.densities = vec![0.0; self.positions.len()];

        // update the lookup table
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
            // scale each particle back up to the drawing size
            let Vec2 { x, y } = *p;
            let xpx = x * SCALE;
            let ypx = y * SCALE;

            // calculate the color using the velocity
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

        // draw the interaction force
        if let Some(mouse) = self.mouse {
            let mpos = (mouse.px / SCALE) - (self.settings.size / SCALE / 2.0);
            let mpos = mpos * SCALE;

            mesh.circle(
                graphics::DrawMode::Stroke(StrokeOptions::default()),
                [mpos.x, mpos.y],
                self.settings.interaction_radius * SCALE,
                0.1,
                graphics::Color::from_rgb(255, 255, 255),
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
    pub fn apply_external_forces(&mut self) {
        self.positions
            // .par_iter()
            .iter()
            .zip(self.velocities.iter_mut())
            .for_each(|(pos, vel)| {
                let accel = Self::external_forces(self.mouse, *pos, *vel, self.settings);
                *vel += accel * self.settings.dtime;
            });
    }

    // use predicted positions rather than actual positions
    // use a constant lookahead time for consistency across TPS variations
    pub fn update_predictions(&mut self) {
        (0..self.len())
            .into_par_iter()
            .map(|i| {
                self.positions[i]
                    + self.velocities[i]
                        * 'lookahead: {
                            #[cfg(not(target_arch = "wasm32"))]
                            break 'lookahead 1. / 120.;
                            #[cfg(target_arch = "wasm32")]
                            break 'lookahead 1. / 60.;
                        }
            })
            .collect_into_vec(&mut self.predictions);
    }

    // precache the densities of all of the particles
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

    // after all the updates, apply the velocities to the positions
    pub fn update_positions(&mut self) {
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

    // make the particles repel
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

    // make the particles resist flow
    pub fn apply_viscosity(&mut self) {
        self.velocities = (0..self.len())
            .into_par_iter()
            .map(|i| {
                let accel = Self::viscosity(
                    &self.predictions,
                    &self.velocities,
                    &self.lookup,
                    self.settings,
                    i,
                );

                self.velocities[i] + accel * self.settings.dtime
            })
            .collect();
    }

    // global update loop
    pub fn update(&mut self) {
        self.apply_external_forces();
        self.update_predictions();
        self.lookup.update(&self.predictions, self.settings);
        self.update_densities();
        self.apply_pressure_forces();
        self.apply_viscosity();
        self.update_positions();
        self.collide();
    }
}

// single particle calculations
// no partial borrowing here is just absurd
// rust, do better
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

    /// smoothing for viscosity. Smooth near the center
    fn viscosity_smoothing(dist: f32, radius: f32) -> f32 {
        if dist >= radius {
            return 0.0;
        }

        let volume = (f32::consts::PI * radius.powi(8)) / 4.0; // calculated by wolfram alpha
        let diff2 = radius.powi(2) - dist.powi(2);
        diff2.powi(3) / volume
    }

    /// the derivative of the smoothing function
    fn smoothing_deriv(dist: f32, radius: f32) -> f32 {
        if dist >= radius || dist == 0.0 {
            return 0.0;
        }

        let scale = 12. / radius.powi(4) * f32::consts::PI;
        (dist - radius) * scale
    }

    /// calculate the density of the scene at a given point.
    /// but, give more weight to the particles closer to the sample point
    fn density(
        positions: &Vec<Vec2>,
        lookup: &SpatialLookup,
        settings: SimSettings,
        sample: Vec2,
    ) -> f32 {
        let raw = rad1(SpatialLookup::pos_to_cell(sample, settings))
            .into_iter()
            .map(|n| SpatialLookup::cell_key(n, settings))
            .flat_map(|key| lookup.get_by_key(key))
            .copied()
            .map(|pidx| {
                let dist = (positions[pidx] - sample).distance(Vec2::ZERO); // get the distance between the two points
                let influence = Self::smoothing(dist, settings.smoothing_radius); // calculate the influence ("weight") of that point
                settings.mass * influence
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
        lookup: &SpatialLookup,
        settings: SimSettings,
        particle: usize,
    ) -> Vec2 {
        let mass = settings.mass;
        let pdensity = densities[particle];
        let ppressure = Self::density_to_pressure(settings, pdensity);

        // use the spatial lookup to find the particles within the smoothing radius
        rad1(SpatialLookup::pos_to_cell(positions[particle], settings))
            .into_iter()
            .map(|n| SpatialLookup::cell_key(n, settings))
            .flat_map(|key| lookup.get_by_key(key))
            .copied()
            .filter(|pidx| pidx != &particle)
            .map(|pidx| {
                let pos = positions[pidx];
                let offset = pos - positions[particle];
                let dist = offset.distance(Vec2::ZERO);

                let dir = if dist == 0. {
                    // shoot off in a random direction. we don't want to have two particles on top of each other
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

    /// calculate mouse and gravity forces
    fn external_forces(
        mouse: Option<MouseState>,
        position: Vec2,
        velocity: Vec2,
        settings: SimSettings,
    ) -> Vec2 {
        let gravity = Vec2::new(0.0, settings.gravity);

        if let Some(mouse) = mouse {
            let mousepos = mouse.px / SCALE - settings.size / SCALE / 2.0;
            let offset = mousepos - position;
            let dist2 = offset.dot(offset);

            if dist2 < settings.interaction_radius.powi(2) {
                let dist = dist2.sqrt();
                let edge = dist / settings.interaction_radius;
                let center = 1. - edge;
                let dir = offset / dist;
                let strength = settings.interaction_strength * mouse.intensity();

                // reduce gravity when interacting with the mouse. makes interaction feel more natural
                let gweight = 1. - (center * (strength / 10.).clamp(0., 1.));

                // the closer you are to mouse, the more you are pulled.
                let accel = gravity * gweight + dir * center * strength;

                return accel - velocity * center;
            }
        }

        gravity
    }

    /// calculate the viscosity force
    fn viscosity(
        positions: &Vec<Vec2>,
        velocities: &Vec<Vec2>,
        lookup: &SpatialLookup,
        settings: SimSettings,
        particle: usize,
    ) -> Vec2 {
        let mut force = Vec2::ZERO;

        let pos = positions[particle];
        let neighbors = rad1(SpatialLookup::pos_to_cell(pos, settings))
            .into_iter()
            .map(|n| SpatialLookup::cell_key(n, settings))
            .flat_map(|key| lookup.get_by_key(key))
            .copied()
            .filter(|pidx| pidx != &particle);

        for pidx in neighbors {
            let other_pos = positions[pidx];
            let dist = pos.distance(other_pos);
            let influence = Self::viscosity_smoothing(dist, settings.smoothing_radius);

            force += (velocities[pidx] - velocities[particle]) * influence;
        }

        force * settings.viscosity_strength
    }
}
