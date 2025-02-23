use lazy_static::lazy_static;
use wgpu::include_spirv;

mod bindings;
pub(super) mod compute;
pub(super) mod vertex;

lazy_static! {
    pub static ref SHADER: wgpu::ShaderModuleDescriptor<'static> =
        include_spirv!(env!("physics.spv"));
}
