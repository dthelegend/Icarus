use std::ffi::CStr;

pub const ENGINE_NAME: &CStr = c"Icarus Engine";
pub const ENGINE_VERSION: u32 = {
    let major = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(0);
    let minor = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0);
    let patch = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0);

    ash::vk::make_api_version(0, major, minor, patch)
};
pub const API_VERSION: u32 = ENGINE_VERSION;
