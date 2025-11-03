use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../physics");

    println!(
        "cargo:rerun-if-changed={}/**/*",
        dir
    );
    println!("cargo:rerun-if-changed=build.rs");
    print!("");

    SpirvBuilder::new(dir, "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .build()?;

    Ok(())
}
