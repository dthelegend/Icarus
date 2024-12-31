use std::sync::Arc;
use log::{error, debug, info};
use thiserror::Error;
use vulkano::{LoadingError, Validated, VulkanError, VulkanLibrary};
use vulkano::device::physical::PhysicalDevice;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::instance::debug::{DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessenger, DebugUtilsMessengerCallback, DebugUtilsMessengerCreateInfo};
use vulkano::swapchain::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::raw_window_handle::HandleError;
use winit::window::{Window, WindowId};
use crate::app::utils::{get_debug_utils_callback, get_required_instance_extensions, get_required_layers, is_required_device_features_support_available, is_required_layer_support_available};

mod utils;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("unable to find required layers")]
    VulkanMissingLayers,
    #[error("unable to find a suitable device")]
    VulkanNoSuitableDevice,
    #[error("window event loop error! {0}")]
    WindowEventError(#[from] EventLoopError),
    #[error("failed to acquire raw window handle! {0}")]
    HandleError(#[from] HandleError),
    #[error("failed to load Vulkan! {0}")]
    LoadingError(#[from] LoadingError),
    #[error("vulkan error! {0}")]
    ValidatedVulkanError(#[from] Validated<VulkanError>),
    #[error("vulkan error! {0}")]
    VulkanError(#[from] VulkanError)
}

// Config
// TODO make this constructible using a builder
pub struct AppConfig {
    app_name: String
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            app_name: String::from("Icarus Engine")
        }
    }
}

// App manager produces instances
pub struct AppManager {
    event_loop: EventLoop<()>,
    vulkan_instance: Arc<Instance>,
}

impl AppManager {
    pub fn from_config(app_config: AppConfig) -> Result<AppManager, AppError> {
        let event_loop = EventLoop::new()?;

        let vk_lib = VulkanLibrary::new()?;

        is_required_layer_support_available(vk_lib.clone())
            .map_err(AppError::from)
            .and_then(|is_supported| is_supported.then_some(()).ok_or(AppError::VulkanMissingLayers))?;
        
        let mut debug_utils_messengers = Vec::new();
        
        #[cfg(debug_assertions)]
        {
            let callback = get_debug_utils_callback();

            let mut create_info = DebugUtilsMessengerCreateInfo::user_callback(callback);
            create_info.message_type = DebugUtilsMessageType::GENERAL | DebugUtilsMessageType::PERFORMANCE | DebugUtilsMessageType::VALIDATION;
            create_info.message_severity = DebugUtilsMessageSeverity::VERBOSE | DebugUtilsMessageSeverity::INFO | DebugUtilsMessageSeverity::WARNING | DebugUtilsMessageSeverity::ERROR;
            
            debug_utils_messengers.push(create_info);
        };
        
        let vulkan_instance = Instance::new(vk_lib, InstanceCreateInfo {
            enabled_extensions: Surface::required_extensions(&event_loop)?.union(&get_required_instance_extensions()),
            enabled_layers: get_required_layers(),
            debug_utils_messengers,
            application_name: Some(app_config.app_name),
            ..InstanceCreateInfo::application_from_cargo_toml()
        })?;

        Ok(AppManager {
            event_loop,
            vulkan_instance
        })
    }

    pub fn run(self) -> Result<(), AppError> {
        let mut handler = AppHandler {
            resources: None,
            vulkan_instance: self.vulkan_instance
        };

        self.event_loop.run_app(&mut handler)?;

        Ok(())
    }
}

struct AppCapabilities {
    rtx: bool
}

impl AppCapabilities {
    fn for_device(device: Arc<PhysicalDevice>) -> Option<Self> {
        is_required_device_features_support_available(device.clone()).then_some(
            AppCapabilities {
                rtx: device.supported_extensions().nv_ray_tracing
            })
    }

    fn score(&self) -> u32 {
        if self.rtx { 1 } else { 0 }
    }
}

struct AppResources {
    window: Arc<Window>,
    vulkan_surface: Arc<Surface>,
    capabilities: AppCapabilities
}

struct AppHandler {
    vulkan_instance: Arc<Instance>,
    resources: Option<AppResources>,
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("TODO Replace with title")
            .with_inner_size(LogicalSize::new(1920, 1080));
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                error!("Failed to create Window! {e}");
                event_loop.exit();
                return;
            }
        };

        debug!("Created a new window!");

        let vulkan_surface = match Surface::from_window(self.vulkan_instance.clone(), window.clone()) {
            Ok(surface) => surface,
            Err(e) => {
                error!("Failed to create a Surface! {e}");
                event_loop.exit();
                return;
            }
        };

        let (vulkan_device, capabilities) = {
            let all_devices = match self.vulkan_instance.enumerate_physical_devices() {
                Ok(devices) => devices,
                Err(e) => {
                    error!("Failed to enumerate physical devices! {e}");
                    event_loop.exit();
                    return;
                }
            };

            let best_device = all_devices
                .filter_map(|physical_device|
                    AppCapabilities::for_device(physical_device.clone())
                        .map(|app_capabilities| (physical_device, app_capabilities)))
                .max_by_key(|(_physical_device, d)| d.score());

            match best_device {
                Some(device) => device,
                None => {
                    error!("Failed to find a suitable physical device!");
                    event_loop.exit();
                    return;
                }
            }
        };
        
        // TODO Create logical device and queues

        self.resources = Some(AppResources {
            vulkan_surface,
            window,
            capabilities
        })
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Received Close Window Event");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.resources.as_ref().unwrap().window.request_redraw();
            }
            _ => (),
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.resources = None;
    }
}
