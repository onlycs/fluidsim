use super::vec2::Length2;
use crate::prelude::*;
use ggez::graphics;
use uom::si::f32::Length;
use vec2::Velocity2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    pub position: Length2,
    pub velocity: Velocity2,
    pub radius: Length,
    pub mass: Mass,
}

impl Particle {
    pub fn new(position: Length2, radius: Length) -> Self {
        Self {
            position,
            velocity: Velocity2::zero(),
            radius,
            mass: Mass::new::<kilogram>(1.0),
        }
    }

    pub fn draw(&self, mesh: &mut graphics::MeshBuilder) -> Result<(), ggez::GameError> {
        let Length2 { x, y } = self.position;
        let xpx = x.get::<pixel>();
        let ypx = y.get::<pixel>();

        trace!(
            "Drawing particle at ({}, {}) from ({:?}, {:?})",
            xpx,
            ypx,
            x,
            y
        );

        mesh.circle(
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            [xpx, ypx],
            self.radius.get::<pixel>(),
            0.1,
            graphics::Color::WHITE,
        )?;

        Ok(())
    }
}
