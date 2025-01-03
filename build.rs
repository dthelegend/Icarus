use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("icarus-shaders", "spirv-unknown-vulkan1.2")
        .print_metadata(MetadataPrintout::Full)
        .build()?;
    Ok(())
}