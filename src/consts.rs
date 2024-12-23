use std::ffi::CStr;

pub const ENGINE_NAME: &CStr = c"Icarus Engine";
pub const ENGINE_VERSION: u32 = {
    const fn str_to_u32(num_str: &str) -> u32 {
        match u32::from_str_radix(num_str, 10) {
            Ok(num) => num,
            Err(_) => panic!("invalid Engine version"),
        }
    }

    let major = str_to_u32(env!("CARGO_PKG_VERSION_MAJOR"));
    let minor = str_to_u32(env!("CARGO_PKG_VERSION_MINOR"));
    let patch = str_to_u32(env!("CARGO_PKG_VERSION_PATCH"));

    ash::vk::make_api_version(0, major, minor, patch)
};
pub const API_VERSION: u32 = ash::vk::make_api_version(0, 1, 0, 0);
