use spirv_builder::{SpirvBuilder, SpirvMetadata};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spirv_crate = concat!(env!("CARGO_MANIFEST_DIR"), "/../physics");

    println!("cargo:rerun-if-changed={}/**/*", spirv_crate);
    println!("cargo:rerun-if-changed=build.rs");

    let mut b = SpirvBuilder::new(spirv_crate, "spirv-unknown-vulkan1.4")
        .spirv_metadata(SpirvMetadata::Full);

    b.build_script.defaults = true;
    b.build_script.env_shader_spv_path = Some(true);
    b.build()?;

    Ok(())
}
