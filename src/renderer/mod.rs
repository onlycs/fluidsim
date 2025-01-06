use crate::prelude::*;
use crate::{physics::PhysicsWorkerThread, vec2::Length2};
use ggez::{
    event,
    graphics::{self, DrawParam},
    input::keyboard::KeyInput,
    winit::keyboard::{KeyCode, PhysicalKey},
};
use panel::Panel;

mod egui_translator;
mod panel;

pub struct State {
    physics: PhysicsWorkerThread,
    panel: Panel,
}

impl State {
    pub fn new() -> Self {
        Self {
            physics: PhysicsWorkerThread::new(),
            panel: Panel::default(),
        }
    }
}

impl event::EventHandler for State {
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        self.panel.update(ctx);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        let (width, height) = ctx.gfx.drawable_size();
        let (halfw, halfh) = (width / 2., height / 2.);

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
        canvas.draw(
            &graphics::Mesh::from_data(ctx, mesh.build()),
            GlamVec2::ZERO,
        );

        // draw the panel to the canvas
        canvas.draw(&*self.panel, DrawParam::new().dest([-halfw, -halfh]));

        canvas.finish(ctx)?;

        ggez::timer::yield_now();

        Ok(())
    }

    fn resize_event(
        &mut self,
        ctx: &mut ggez::Context,
        width: f32,
        height: f32,
    ) -> Result<(), ggez::GameError> {
        let Some(wpos) = self.panel.update_wpos(ctx)? else {
            return Ok(());
        };

        let wsize = Length2::new::<pixel>(width, height);

        self.panel.set_window(wsize, wpos);

        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        input: KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        let PhysicalKey::Code(kc) = input.event.physical_key else {
            return Ok(());
        };

        match kc {
            KeyCode::Space => ipc::physics_send(ToPhysics::Pause),
            KeyCode::ArrowRight => ipc::physics_send(ToPhysics::Step),
            KeyCode::KeyR => ipc::physics_send(ToPhysics::Reset),
            KeyCode::KeyC => {
                debug!("Toggling config panel");
                self.panel.toggle();
            }
            KeyCode::KeyH => {
                debug!("Toggling help text");
                self.panel.toggle_help();
            }
            KeyCode::KeyQ if input.mods.control_key() => {
                info!("Got ctrl+q, quitting!");
                ctx.request_quit();
            }
            _ => (),
        }

        Ok(())
    }
}
