pub use ggez::glam::*;
use ggez::graphics;

use super::PXSCALE;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
}

impl Particle {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
        }
    }

    pub fn draw(&self, mesh: &mut graphics::MeshBuilder) -> Result<(), ggez::GameError> {
        let Vec2 { x, y } = self.position;
        let xpx = x * PXSCALE;
        let ypx = y * PXSCALE;

        trace!("Drawing particle at ({}, {}) from ({}, {})", xpx, ypx, x, y);

        mesh.circle(
            graphics::DrawMode::Fill(graphics::FillOptions::default()),
            [xpx, ypx],
            7.5,
            0.1,
            graphics::Color::WHITE,
        )?;

        Ok(())
    }
}
