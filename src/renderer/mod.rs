use std::time::Instant;

use async_std::channel::{self, Sender};
use ggez::{
    event,
    glam::Vec2,
    graphics::{self, Color, StrokeOptions},
    mint::Point2,
    timer::{self, TimeContext},
};
use message::RendererMessage;

use crate::physics::PhysicsWorkerThread;

pub mod message;

pub struct State {
    physics: PhysicsWorkerThread,
    messenger: Sender<RendererMessage>,
}

impl State {
    pub fn new() -> Self {
        let (tx, rx) = channel::unbounded();

        Self {
            physics: PhysicsWorkerThread::new(600.0, 400.0, rx),
            messenger: tx,
        }
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        // todo
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        let sc = self.physics.get();

        let (width, height) = ctx.gfx.drawable_size();
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);
        canvas.set_screen_coordinates(graphics::Rect::new(
            -width / 2.0,
            -height / 2.0,
            width,
            height,
        ));

        let mut mesh = graphics::MeshBuilder::new();

        sc.particles.iter().for_each(|p| p.draw(&mut mesh).unwrap());

        canvas.draw(&graphics::Mesh::from_data(ctx, mesh.build()), Vec2::ZERO);
        canvas.finish(ctx)?;

        ggez::timer::yield_now();

        // todo
        Ok(())
    }

    fn resize_event(
        &mut self,
        _ctx: &mut ggez::Context,
        width: f32,
        height: f32,
    ) -> Result<(), ggez::GameError> {
        self.messenger
            .send_blocking(RendererMessage::Resize(width, height))
            .map_err(|_| ggez::GameError::ResourceLoadError("Failed to send resize message".into()))
    }
}
