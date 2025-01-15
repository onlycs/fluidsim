pub mod panel;

use crate::{physics::PhysicsWorkerThread, prelude::*};

use bevy::{render::renderer::RenderDevice, window::PrimaryWindow};
use bevy_egui::EguiContexts;
use egui::{RichText, Slider};
use physics::scene;

#[derive(Component)]
pub struct ParticleMarker(usize);

pub struct PanelConfig {
    pub enabled: bool,
    pub help: bool,
}

impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            help: true,
        }
    }
}

#[derive(Resource)]
pub struct GraphicsState {
    physics: PhysicsWorkerThread,
    panel_cfg: PanelConfig,
    physics_cfg: SimSettings,
    mouse: MouseState,
}

pub fn resources(mut commands: Commands) {
    // insert a new state
    commands.insert_resource(GraphicsState {
        physics: PhysicsWorkerThread::new(),
        panel_cfg: PanelConfig::default(),
        physics_cfg: SimSettings::default(),
        mouse: MouseState::default(),
    });
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    state: Res<GraphicsState>,
    gpu: Res<RenderDevice>,
) {
    commands.spawn((Camera2d, Msaa::Sample2));

    let Vec2 { x, y } = state.physics_cfg.particles;
    let num_particles = x as usize * y as usize;
    let radius = state.physics_cfg.radius * scene::SCALE;

    let circle = meshes.add(Circle::new(radius));
    let col = materials.add(Color::WHITE);

    for i in 0..num_particles {
        commands.spawn((
            ParticleMarker(i),
            Transform::from_xyz(0., 0., 0.),
            Mesh2d(circle.clone()),
            MeshMaterial2d(col.clone()),
        ));
    }
}

pub fn mouse(
    mut state: ResMut<GraphicsState>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    if let Some(px) = window.single().cursor_position() {
        let left = mouse_btn.just_pressed(MouseButton::Left);
        let right = mouse_btn.just_pressed(MouseButton::Right);
        let data = MouseState {
            px,
            left,
            right,
            ..state.mouse
        };

        if data != state.mouse {
            state.mouse = data;
            ipc::physics_send(ToPhysics::UpdateMouse(data));
        }
    }
}

pub fn draw(
    mut state: ResMut<GraphicsState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut particles: Query<
        (
            &mut Transform,
            &MeshMaterial2d<ColorMaterial>,
            &ParticleMarker,
        ),
        With<ParticleMarker>,
    >,
) {
    let scene = state.physics.get();
    let max_speed = 15.0;
    let g = LinearGradient::new(vec![
        (0.062, Color::srgb_u8(23, 71, 162)),
        (0.48, Color::srgb_u8(81, 252, 147)),
        (0.65, Color::srgb_u8(252, 237, 6)),
        (1.0, Color::srgb_u8(239, 74, 12)),
    ]);

    for (transform, mat, marker) in particles.iter_mut() {
        let ParticleMarker(id) = marker;

        let pos = scene.positions[*id] * scene::SCALE;
        let vel = scene.velocities[*id];

        let speed = vel.distance(Vec2::ZERO);
        let relative = speed / max_speed;
        let color = g.sample(relative.min(1.0));

        let mat_handle = &mat.0;

        materials.get_mut(mat_handle).unwrap().color = color;
        *transform.into_inner() = Transform::from_xyz(pos.x, -pos.y, 0.0);
    }
}

// impl State {
//     pub fn new() -> Self {
//         Self {
//             physics: PhysicsWorkerThread::new(),
//             panel: Panel::default(),
//             mouse: None,
//             tps_data: (0.0, 0),
//         }
//     }
// }

// impl event::EventHandler for State {
//     /// Update the panel (mouse/keyboard) as well as sending good mouse data
//     fn update(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
//         let propagate = !self.panel.update(ctx);

//         let mouse = &ctx.mouse;
//         let left_pressed = mouse.button_pressed(MouseButton::Left) && propagate;
//         let any_pressed = (mouse.button_pressed(MouseButton::Right) || left_pressed) && propagate;
//         let data = any_pressed.then_some(MouseState {
//             px: mouse.position().into(),
//             is_left: left_pressed,
//         });

//         if data != self.mouse {
//             self.mouse = data;
//             ipc::physics_send(ToPhysics::UpdateMouse(data));
//         }

//         Ok(())
//     }

//     fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), ggez::GameError> {
//         let (width, height) = ctx.gfx.drawable_size();
//         let (halfw, halfh) = (width / 2., height / 2.);

//         // create and setup the canvas
//         let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLACK);

//         // make the center at zero,zero to make my life easier
//         canvas.set_screen_coordinates(graphics::Rect::new(
//             -width / 2.0,
//             -height / 2.0,
//             width,
//             height,
//         ));

//         // grab the current scene and create a mesh
//         let sc = self.physics.get();
//         let mut mesh = graphics::MeshBuilder::new();

//         // draw to mesh from scene
//         sc.draw(&mut mesh)?;

//         // draw the mesh to the canvas
//         canvas.draw(&graphics::Mesh::from_data(ctx, mesh.build()), Vec2::ZERO);

//         // draw the panel to the canvas
//         canvas.draw(&*self.panel, DrawParam::new().dest([-halfw, -halfh]));

//         // draw the current fps
//         let (ref mut old_tps, ref mut old_count) = self.tps_data;

//         let fps = format!("Rendering FPS: {:.2}", ctx.time.fps());
//         let physics_fps = format!(
//             "Physics TPS: {}",
//             if *old_count >= 10 {
//                 *old_tps = self.physics.tps();
//                 *old_count = 0;
//                 self.physics.tps()
//             } else {
//                 *old_count += 1;
//                 *old_tps
//             }
//         );

//         let fps_text = graphics::Text::new(fps);
//         let physics_fps_text = graphics::Text::new(physics_fps);

//         let fps_dest = Vec2::new(-halfw + 10.0, halfh - 20.0);
//         let physics_fps_dest = Vec2::new(-halfw + 10.0, halfh - 40.0);

//         canvas.draw(&fps_text, fps_dest);
//         canvas.draw(&physics_fps_text, physics_fps_dest);

//         canvas.finish(ctx)?;

//         ggez::timer::yield_now();

//         Ok(())
//     }

//     fn resize_event(
//         &mut self,
//         ctx: &mut ggez::Context,
//         width: f32,
//         height: f32,
//     ) -> Result<(), ggez::GameError> {
//         let Some(wpos) = self.panel.update_wpos(ctx)? else {
//             return Ok(());
//         };

//         let wsize = Vec2::new(width, height);
//         self.panel.set_window(wsize, wpos);

//         Ok(())
//     }

//     fn key_down_event(
//         &mut self,
//         ctx: &mut ggez::Context,
//         input: KeyInput,
//         _repeated: bool,
//     ) -> Result<(), ggez::GameError> {
//         let PhysicalKey::Code(kc) = input.event.physical_key else {
//             return Ok(());
//         };

//         match kc {
//             KeyCode::Space => ipc::physics_send(ToPhysics::Pause),
//             KeyCode::ArrowRight => ipc::physics_send(ToPhysics::Step),
//             KeyCode::KeyR => ipc::physics_send(ToPhysics::Reset),
//             KeyCode::KeyC => {
//                 debug!("Toggling config panel");
//                 self.panel.toggle();
//             }
//             KeyCode::KeyH => {
//                 debug!("Toggling help text");
//                 self.panel.toggle_help();
//             }
//             KeyCode::KeyQ if input.mods.control_key() => {
//                 info!("Got ctrl+q, quitting!");
//                 ipc::physics_send(ToPhysics::Kill);
//                 ctx.request_quit();
//             }
//             _ => (),
//         }

//         Ok(())
//     }
// }
