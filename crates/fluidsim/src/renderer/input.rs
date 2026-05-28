use std::{collections::HashMap, iter, time::Instant};

use glam::{Vec2, vec2};
use winit::{
    event::{KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Default)]
pub(crate) struct InputProcessor {
    pub(crate) keys: HashMap<KeyCode, Instant>,

    mouse_pos: Vec2,
    lmb: bool,
    rmb: bool,
}

pub(crate) enum HumanInput {
    None,
    Keyboard {
        ui: Vec<KeyCode>,
        motion: Vec<KeyCode>,
    },
    Mouse {
        position: Vec2,
        lmb: bool,
        rmb: bool,
    },
}

impl InputProcessor {
    pub(crate) fn process(&mut self, event: &WindowEvent) -> HumanInput {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state,
                        repeat: false,
                        ..
                    },
                is_synthetic: _,
            } => {
                if state.is_pressed() {
                    self.keys.insert(*key, Instant::now());
                    HumanInput::Keyboard {
                        ui: self.ui_keys().chain(iter::once(*key)).collect(),
                        motion: self.keys.keys().copied().collect(),
                    }
                } else {
                    self.keys.remove(key);
                    HumanInput::Keyboard {
                        ui: self.ui_keys().collect(),
                        motion: self.keys.keys().copied().collect(),
                    }
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position: pos,
            } => {
                self.mouse_pos = vec2(pos.x as f32, pos.y as f32);

                HumanInput::Mouse {
                    position: self.mouse_pos,
                    lmb: self.lmb,
                    rmb: self.rmb,
                }
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

                HumanInput::Mouse {
                    position: self.mouse_pos,
                    lmb: self.lmb,
                    rmb: self.rmb,
                }
            }
            _ if self.keys.is_empty() => HumanInput::None,
            _ => HumanInput::Keyboard {
                ui: self.ui_keys().collect(),
                motion: self.keys.keys().copied().collect(),
            },
        }
    }

    fn ui_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.keys
            .iter()
            .filter(|(_, t)| t.elapsed().as_millis() > 350)
            .map(|(k, _)| *k)
    }
}
