use wgpu::include_spirv;

mod buffers;
pub mod compute;
pub mod pipelines;
pub(super) mod vertex;

pub const SHADER: wgpu::ShaderModuleDescriptor<'static> = include_spirv!(env!("physics.spv"));
