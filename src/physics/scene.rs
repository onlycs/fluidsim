use super::particle::Particle;
use super::PXSCALE;
use ggez::glam::Vec2;

#[derive(Clone, Debug)]
pub struct Scene {
    pub particles: Vec<Particle>,
    pub(crate) width: f32,
    pub(crate) height: f32,
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
        }
    }

    pub fn update(&mut self) {
        // todo
    }
}
