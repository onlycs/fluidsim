use std::collections::HashSet;

use glam::{Vec2, vec2};
use gpu_shared::MouseState;
use winit::{
    event::{Modifiers, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::renderer::shader::compute::PhysicsShader;

#[derive(Default)]
pub(crate) struct InputProcessor {
    pub(crate) keys: HashSet<KeyCode>,
    pub(crate) mods: Modifiers,

    mouse_pos: Vec2,
    lmb: bool,
    rmb: bool,
}

pub(crate) enum HumanInput {
    None,
    Keyboard,
    Mouse,
}

impl InputProcessor {
    pub(crate) fn process(&mut self, event: &WindowEvent) -> HumanInput {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let PhysicalKey::Code(key) = event.physical_key else {
                    return HumanInput::None;
                };

                if event.state.is_pressed() {
                    self.keys.insert(key);
                } else {
                    self.keys.remove(&key);
                }

                HumanInput::Keyboard
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.keys.clear();
                self.mods = *mods;

                HumanInput::Keyboard
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position: pos,
            } => {
                self.mouse_pos = vec2(pos.x as f32, pos.y as f32);

                HumanInput::Mouse
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                match button {
                    MouseButton::Left => self.lmb = state.is_pressed(),
                    MouseButton::Right => self.rmb = state.is_pressed(),
                    _ => return HumanInput::None,
                }

                HumanInput::Mouse
            }
            _ => HumanInput::None,
        }
    }

    pub(crate) fn write_mouse(&self, physics: &mut PhysicsShader) {
        physics.set_mouse(MouseState::new(self.mouse_pos, self.lmb, self.rmb));
    }

    pub(crate) fn keydown(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.keys.iter().copied()
    }
}
