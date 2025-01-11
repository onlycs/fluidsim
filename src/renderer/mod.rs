use crate::physics::PhysicsWorkerThread;
use crate::prelude::*;
use ggez::{
    event,
    graphics::{self, DrawParam},
    input::keyboard::KeyInput,
    winit::{
        event::MouseButton,
        keyboard::{KeyCode, PhysicalKey},
    },
};
use panel::Panel;
use physics::settings::MouseState;

mod egui_translator;
mod panel;

pub struct State {
    physics: PhysicsWorkerThread,
    panel: Panel,
    mouse: Option<MouseState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            physics: PhysicsWorkerThread::new(),
            panel: Panel::default(),
            mouse: None,
        }
    }
}

impl event::EventHandler for State {
    /// Update the panel (mouse/keyboard) as well as sending good mouse data
    fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
        let propagate = !self.panel.update(ctx);

        let mouse = &ctx.mouse;
        let left_pressed = mouse.button_pressed(MouseButton::Left) && propagate;
        let any_pressed = (mouse.button_pressed(MouseButton::Right) || left_pressed) && propagate;
        let data = any_pressed.then_some(MouseState {
            px: mouse.position().into(),
            is_left: left_pressed,
        });

        if data != self.mouse {
            self.mouse = data;
            ipc::physics_send(ToPhysics::UpdateMouse(data));
        }

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
        sc.draw(&mut mesh)?;

        // draw the mesh to the canvas
        canvas.draw(&graphics::Mesh::from_data(ctx, mesh.build()), Vec2::ZERO);

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

        let wsize = Vec2::new(width, height);
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
                ipc::physics_send(ToPhysics::Kill);
                ctx.request_quit();
            }
            _ => (),
        }

        Ok(())
    }
}
