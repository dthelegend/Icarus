use std::sync::Arc;
use vulkano::device::Device;
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};
use vulkano::{Validated, VulkanError};

const SHADER_MODULE_BIN: &[u32] = &{
    // All this song and dance to guarantee that this constant is aligned
    // to a 32 bit boundary at compile time! This should be in core!
    const RAW_BYTES: &[u8] = include_bytes!(env!("icarus_shaders.spv"));
    const RAW_BYTES_LEN: usize = RAW_BYTES.len();
    const U32_BYTES: usize = RAW_BYTES_LEN / 4;

    // NB This is only legal because valid SPIRV uses u32 words
    assert!(RAW_BYTES_LEN == U32_BYTES * 4);

    let mut u32_buffer = [0; U32_BYTES];
    let mut idx = 0;
    while idx < U32_BYTES {
        let chunk_idx = idx * 4;
        u32_buffer[idx] = u32::from_ne_bytes([RAW_BYTES[chunk_idx + 0], RAW_BYTES[chunk_idx + 1], RAW_BYTES[chunk_idx + 2], RAW_BYTES[chunk_idx + 3]]);
        idx += 1;
    }

    u32_buffer
};

struct IcarusShader {
    shader_module: Arc<ShaderModule>,
}

impl IcarusShader {
    pub fn load(logical_device: Arc<Device>, ) -> Result<IcarusShader, Validated<VulkanError>> {
        unsafe { ShaderModule::new(logical_device, ShaderModuleCreateInfo::new(SHADER_MODULE_BIN)) }.map(|shader_module| IcarusShader { shader_module} )
    }

    pub fn graphics_pipeline(&self) {
        // self.shader_module.entry_point("main")?.module().
        todo!()
    }
}

