#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimSettings {
    pub gravity: f32,
}

impl Default for SimSettings {
    fn default() -> Self {
        Self { gravity: -9.8 }
    }
}
