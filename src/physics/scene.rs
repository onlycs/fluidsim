use super::particle::Particle;
use super::settings::SimSettings;
use super::PXSCALE;
use ggez::glam::Vec2;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

const PHYSICS_SCALE: f32 = PXSCALE * 0.5;

#[derive(Clone, Debug)]
pub struct Scene {
    pub particles: Vec<Particle>,
    pub(super) settings: SimSettings,
    pub(super) width: f32,
    pub(super) height: f32,
}

impl Scene {
    pub fn new(widthpx: f32, heightpx: f32) -> Self {
        let width = widthpx / 2.0 / PXSCALE;
        let height = heightpx / 2.0 / PXSCALE;

        Self {
            particles: (0..20)
                .flat_map(|i| {
                    let i = i as f32 - 10.0;

                    (0..20).map(move |j| {
                        let j = j as f32 - 10.0;
                        Particle::new(Vec2::new(i, j))
                    })
                })
                .collect(),

            width,
            height,
            settings: SimSettings::default(),
        }
    }

    pub fn update(&mut self) {
        let spt = 1.0 / self.settings.tps as f32;
        let g = self.settings.gravity * PHYSICS_SCALE;

        // vf = vi + at
        let at = spt * g;
        self.particles
            .par_iter_mut()
            .for_each(|p| p.velocity += Vec2::new(0.0, -at)); // -at because gravity

        // d = vt
        self.particles
            .par_iter_mut()
            .for_each(|p| p.position += p.velocity * spt);
    }
}
