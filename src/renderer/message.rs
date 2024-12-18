#[derive(Clone, Debug, PartialEq)]
pub enum RendererMessage {
    Resize(f32, f32),
}
