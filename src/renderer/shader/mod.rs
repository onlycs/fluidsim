pub(super) mod compute;
pub(super) mod vertex;

pub const FS: wgpu::ShaderModuleDescriptor<'_> = wgpu::include_wgsl!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/shaders/circle.fs.wgsl"
));

pub const VS: wgpu::ShaderModuleDescriptor<'_> = wgpu::include_wgsl!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/shaders/circle.vs.wgsl"
));

pub const CS: wgpu::ShaderModuleDescriptor<'_> = wgpu::include_wgsl!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/shaders/physics.comp.wgsl"
));

pub const ARRAY_LEN: usize = 16384;
