use egui::ahash::HashSet;
use winit::{
    event::{Modifiers, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Default)]
pub struct InputHelper {
    pub keys: HashSet<KeyCode>,
    pub mods: Modifiers,

    pub mouse_pos: (f32, f32),
    pub lmb: bool,
    pub rmb: bool,
}

pub enum InputResponse {
    None,
    Keyboard,
    Mouse,
}

impl InputHelper {
    pub fn process(&mut self, event: &WindowEvent) -> InputResponse {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let PhysicalKey::Code(key) = event.physical_key else {
                    return InputResponse::None;
                };

                if event.state.is_pressed() {
                    self.keys.insert(key);
                } else {
                    self.keys.remove(&key);
                }

                InputResponse::Keyboard
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.keys.clear();
                self.mods = *mods;

                InputResponse::Keyboard
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position: pos,
            } => {
                self.mouse_pos = (*pos).into();

                InputResponse::Mouse
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                match button {
                    MouseButton::Left => self.lmb = state.is_pressed(),
                    MouseButton::Right => self.rmb = state.is_pressed(),
                    _ => return InputResponse::None,
                }

                InputResponse::Mouse
            }
            _ => InputResponse::None,
        }
    }

    pub fn keydown<'a>(&'a self) -> impl Iterator<Item = KeyCode> + 'a {
        self.keys.iter().copied()
    }
}
