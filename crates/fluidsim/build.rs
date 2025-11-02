use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "cargo:rerun-if-changed={}/../physics/**/*",
        env!("CARGO_MANIFEST_DIR")
    );
    println!("cargo:rerun-if-changed=build.rs");
    print!("");

    SpirvBuilder::new("../physics", "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}
