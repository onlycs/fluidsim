use egui::ahash::HashSet;
use winit::{
    event::{Modifiers, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Default)]
pub struct KeyboardHelper {
    pub keys: HashSet<KeyCode>,
    pub mods: Modifiers,
}

impl KeyboardHelper {
    pub fn process(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                let PhysicalKey::Code(key) = event.physical_key else {
                    return false;
                };

                if event.state.is_pressed() {
                    self.keys.insert(key);
                } else {
                    self.keys.remove(&key);
                }

                true
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.keys.clear();
                self.mods = *mods;

                true
            }
            _ => false,
        }
    }

    pub fn keydown<'a>(&'a self) -> impl Iterator<Item = KeyCode> + 'a {
        self.keys.iter().copied()
    }
}
