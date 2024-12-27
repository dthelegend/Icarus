use ash::prelude::VkResult;
use log::{debug, log, log_enabled, warn};
use std::ffi::{c_void, CStr};
use std::ptr;
use ash::vk;
use ash::vk::{DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCreateInfoEXT};
use log::Level;
use thiserror::Error;
use winit::raw_window_handle::{DisplayHandle, WindowHandle};

#[derive(Debug, Error)]
pub enum VulkanError {
    #[error("Internal vulkan error {0}")]
    VulkanInternalError(#[from] vk::Result),
    #[error("FFI error {0}")]
    FFIError(#[from] std::ffi::FromBytesUntilNulError),
}

pub(crate) struct VulkanInstance{
    entry: ash::Entry,
    instance: ash::Instance
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_type: DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message).to_string_lossy();

    let log_level = match message_severity {
        DebugUtilsMessageSeverityFlagsEXT::VERBOSE => Level::Trace,
        DebugUtilsMessageSeverityFlagsEXT::WARNING => Level::Warn,
        DebugUtilsMessageSeverityFlagsEXT::INFO => Level::Debug, // Internal vulkan stuff is debug at most
        _ => Level::Error
    };


    let target = match message_type {
        DebugUtilsMessageTypeFlagsEXT::GENERAL => "vulkan::general",
        DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "vulkan::performance",
        DebugUtilsMessageTypeFlagsEXT::VALIDATION => "vulkan::validation",
        _ => "vulkan::spooky",
    };

    log!(target: target, log_level, "{}", message);

    vk::FALSE
}

impl VulkanInstance {
    pub(crate) fn create(display_handle: &DisplayHandle, app_info: vk::ApplicationInfo) -> Result<Self, VulkanError> {
        let ash_entry = ash::Entry::linked();

        let surface_extensions = ash_window::enumerate_required_extensions(display_handle.as_raw())?;

        let mut create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info);

        let vulkan_instance = if cfg!(debug_assertions) && Self::is_validation_layer_support_available(&ash_entry)? {
            // Check if all required validation layers are available
            debug!("Validation layer support is available");

            let mut surface_extensions_plus = Vec::from(surface_extensions);
            surface_extensions_plus.extend(Self::REQUIRED_VALIDATION_LAYERS.into_iter().map(CStr::as_ptr));
            let surface_extensions_plus_boxed = surface_extensions_plus.into_boxed_slice();
            let mut debug_msg_info = Self::create_debug_msg_info();

            create_info = create_info
                .push_next(&mut debug_msg_info)
                .enabled_extension_names(&surface_extensions_plus_boxed);

            unsafe { ash_entry.create_instance(&create_info, None) }?
        } else {
            if cfg!(debug_assertions) {
                warn!("Validation layer support is not available");
            }
            create_info = create_info
                .enabled_extension_names(surface_extensions);

            unsafe { ash_entry.create_instance(&create_info, None) }?
        };

        let instance = Self {
            entry: ash_entry,
            instance: vulkan_instance,
        };

        Ok(instance)
    }

    const REQUIRED_VALIDATION_LAYERS: [&'static CStr; 1] = [
        c"VK_LAYER_KHRONOS_validation"
    ];

    fn create_debug_msg_info<'a>() -> DebugUtilsMessengerCreateInfoEXT<'a> {
        let severity = {
            DebugUtilsMessageSeverityFlagsEXT::WARNING
                | DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | DebugUtilsMessageSeverityFlagsEXT::INFO
                | DebugUtilsMessageSeverityFlagsEXT::ERROR
        };

        let message_type = {
            DebugUtilsMessageTypeFlagsEXT::GENERAL
                | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | DebugUtilsMessageTypeFlagsEXT::VALIDATION
        };

        DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(severity)
            .message_type(message_type)
            .pfn_user_callback(Some(vulkan_debug_utils_callback))
    }

    fn is_validation_layer_support_available(entry: &ash::Entry) -> Result<bool, VulkanError> {
        // if support validation layer, then return true
        let layer_property_list = unsafe { entry
            .enumerate_instance_layer_properties()? };

        if layer_property_list.len() <= 0 {
            return Ok(false);
        }

        let layer_property_cstr_list = layer_property_list
            .into_iter()
            .map(|layer| unsafe {
                CStr::from_ptr(layer.layer_name.as_ptr()).to_owned()
            })
            .collect::<Vec<_>>();

        debug!("Available Layers:\n - {0}", layer_property_cstr_list.iter().map(|x| x.to_string_lossy()).collect::<Vec<_>>().join("\n - "));

        Ok(Self::REQUIRED_VALIDATION_LAYERS.into_iter().all(|x| {
            layer_property_cstr_list.iter().any(|y| x.eq(y))
        }))
    }

    pub(crate) fn create_surface(&self, display_handle: &DisplayHandle, window_handle: &WindowHandle) -> Result<vk::SurfaceKHR, VulkanError> {
        Ok(unsafe {
            ash_window::create_surface(
                &self.entry,
                &self.instance,
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
        }?)
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None); }
    }
}