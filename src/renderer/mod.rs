use crate::physics::PhysicsWorkerThread;
use crate::prelude::*;
use ggez::{
    event,
    glam::Vec2,
    graphics::{self},
};

pub struct State {
    physics: PhysicsWorkerThread,
}

impl State {
    pub fn new() -> Self {
        Self {
            physics: PhysicsWorkerThread::new(600.0, 400.0),
        }
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        // unnecessary, work is done on a thread!
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        let (width, height) = ctx.gfx.drawable_size();

        // create and setup the canvas
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

        // make the center at zero,zero to make my life easier
        canvas.set_screen_coordinates(graphics::Rect::new(
            -width / 2.0,
            -height / 2.0,
            width,
            height,
        ));

        // grab the current scene and create a mesh
        let sc = self.physics.get();
        let mut mesh = graphics::MeshBuilder::new();

        // draw to mesh from scene
        sc.particles.iter().for_each(|p| p.draw(&mut mesh).unwrap());

        // draw the mesh to the canvas
        canvas.draw(&graphics::Mesh::from_data(ctx, mesh.build()), Vec2::ZERO);
        canvas.finish(ctx)?;

        ggez::timer::yield_now();

        Ok(())
    }

    fn resize_event(
        &mut self,
        _ctx: &mut ggez::Context,
        width: f32,
        height: f32,
    ) -> Result<(), ggez::GameError> {
        ipc::physics_send(ToPhysics::Resize(width, height));
        Ok(())
    }
}
