use lazy_static::lazy_static;
use wgpu::include_spirv;

pub(super) mod compute;
pub(super) mod vertex;

pub const ARRAY_LEN: usize = 16384;

lazy_static! {
    pub static ref SHADER: wgpu::ShaderModuleDescriptor<'static> =
        include_spirv!(env!("physics.spv"));
}
