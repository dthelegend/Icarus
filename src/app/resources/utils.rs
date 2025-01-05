use log::{debug, log, Level};
use std::sync::Arc;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{DeviceExtensions, DeviceFeatures};
use vulkano::instance::debug::{DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessengerCallback, DebugUtilsMessengerCallbackData, DebugUtilsMessengerCreateInfo};
use vulkano::instance::InstanceExtensions;
use vulkano::{VulkanError, VulkanLibrary};

pub const REQUIRED_INSTANCE_EXTENSIONS: InstanceExtensions = InstanceExtensions {
    khr_surface: true,
    ext_debug_utils: cfg!(debug_assertions),
    ..InstanceExtensions::empty()
};

const REQUIRED_LAYERS: &[&str] = &[
    #[cfg(debug_assertions)]
    "VK_LAYER_KHRONOS_validation",
];

pub fn get_required_layers() -> Vec<String> {
    REQUIRED_LAYERS.into_iter().map(|&x| String::from(x)).collect()
}


pub fn get_debug_utils_callback() -> Arc<DebugUtilsMessengerCallback> {
    unsafe { DebugUtilsMessengerCallback::new(vulkan_debug_utils_callback) }
}

fn vulkan_debug_utils_callback(
    message_severity: DebugUtilsMessageSeverity,
    message_type: DebugUtilsMessageType,
    data: DebugUtilsMessengerCallbackData<'_>
) {
    let log_level = match message_severity {
        DebugUtilsMessageSeverity::VERBOSE => Level::Trace,
        DebugUtilsMessageSeverity::WARNING => Level::Warn,
        DebugUtilsMessageSeverity::INFO => Level::Debug, // Internal vulkan stuff is debug at most
        _ => Level::Error
    };

    let target = match message_type {
        DebugUtilsMessageType::GENERAL => "vulkan::general",
        DebugUtilsMessageType::PERFORMANCE => "vulkan::performance",
        DebugUtilsMessageType::VALIDATION => "vulkan::validation",
        _ => "vulkan::unknown"
    };

    log!(target: target, log_level, "{}", data.message)
}

pub fn is_required_layer_support_available(vk_lib: &Arc<VulkanLibrary>) -> Result<bool, VulkanError> {
    // if support validation layer, then return true
    let layer_property_list: Vec<_> = vk_lib.layer_properties()?.collect();

    if layer_property_list.len() <= 0 {
        return Ok(false);
    }

    debug!("Available Layers:\n - {0}", layer_property_list.iter().map(|x| x.name()).collect::<Vec<_>>().join("\n - "));

    Ok(REQUIRED_LAYERS.into_iter().all(|&x| {
        layer_property_list.iter().any(|y| y.name() == x)
    }))
}
