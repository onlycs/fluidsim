use core::f32;

use crate::prelude::*;
use ggez::graphics;
use vec2::Length2;

use super::particle::Particle;
use super::settings::SimSettings;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

#[derive(Clone, Debug)]
pub struct Scene {
    pub particles: Vec<Particle>,
    pub(super) settings: SimSettings,
}

// creation and updating scene settings, etc
impl Scene {
    pub fn new() -> Self {
        let settings = SimSettings::default();

        let mut this = Self {
            particles: Vec::new(),
            settings,
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

        self.particles.clear();

        for i in 0..(nx as i32) {
            for j in 0..(ny as i32) {
                let offset = Length2::of((size + gap) * i as f32, -(size + gap) * j as f32);
                let pos = tl + offset;
                let p = Particle::new(pos, self.settings.radius);

                self.particles.push(p);
            }
        }
    }

    pub fn update_settings(&mut self, settings: SimSettings) {
        if self.settings.radius != settings.radius {
            self.particles
                .par_iter_mut()
                .for_each(|p| p.radius = settings.radius);
        }

        self.settings = settings;
    }
}

// actual physics
impl Scene {
    pub fn apply_gravity(&mut self) {
        // vf = vi + at
        let at = self.settings.gravity * self.settings.tick_delay;
        self.particles.par_iter_mut().for_each(|p| p.velocity += at);
    }

    pub fn update_positions(&mut self) {
        // d = vt
        self.particles
            .par_iter_mut()
            .for_each(|p| p.position += p.velocity * self.settings.tick_delay);
    }

    pub fn collide(&mut self) {
        let Length2 { x, y } = self.absbounds();

        self.particles.par_iter_mut().for_each(|p| {
            if (p.position.y.abs() + p.radius) > y {
                let sign = p.position.y.get::<meter>().signum();

                p.position.y = y * sign + p.radius * -sign;
                p.velocity.y *= -self.settings.collision_dampening;
            }

            if (p.position.x.abs() + p.radius) > x {
                let sign = p.position.x.get::<meter>().signum();

                p.position.x = x * sign + p.radius * -sign;
                p.velocity.x *= -self.settings.collision_dampening;
            }
        });
    }

    pub fn draw(&self, mesh: &mut graphics::MeshBuilder) -> Result<(), ggez::GameError> {
        for p in &self.particles {
            p.draw(mesh)?;
        }

        Ok(())
    }

    pub fn update(&mut self) {
        self.apply_gravity();
        self.update_positions();

        self.collide();
    }
}

// density and pressure calculations
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

    fn density(&self, sample: Length2) -> f32 {
        self.particles
            .par_iter()
            .map(|particle| {
                let dist = (particle.position - sample).mag();
                let influence = Self::smoothing(dist, self.settings.smoothing_radius);
                particle.mass.get::<kilogram>() * influence
            })
            .sum()
    }
}
