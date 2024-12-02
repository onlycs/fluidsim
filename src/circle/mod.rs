use std::borrow::Cow;

pub const DESCRIPTOR: wgpu::ShaderModuleDescriptor = wgpu::ShaderModuleDescriptor {
    label: Some("Circle Shader"),
    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/circle.wgsl"))),
};
