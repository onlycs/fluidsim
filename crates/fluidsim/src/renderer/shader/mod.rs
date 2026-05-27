use std::sync::OnceLock;

use wgpu::include_spirv;

pub(super) mod circles;
pub mod lines;
pub mod physics;
pub mod pipelines;

pub(crate) fn shader_module(device: &wgpu::Device) -> &wgpu::ShaderModule {
    const SHADER: wgpu::ShaderModuleDescriptor<'static> = include_spirv!(env!("physics.spv"));
    static MODULE: OnceLock<wgpu::ShaderModule> = OnceLock::new();

    MODULE.get_or_init(|| device.create_shader_module(SHADER))
}
