use std::cmp::{max, min};
use std::sync::Arc;
use log::{error, debug, info};
use thiserror::Error;
use vulkano::{LoadingError, Validated, VulkanError, VulkanLibrary};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo, QueueFamilyProperties, QueueFlags};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::format::Format;
use vulkano::image::{Image, ImageUsage};
use vulkano::swapchain::{ColorSpace, Swapchain, SwapchainCreateInfo};
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
use crate::app::utils::{get_debug_utils_callback, get_required_device_extensions, get_required_device_features, get_required_instance_extensions, get_required_layers, is_required_device_support_available, is_required_layer_support_available};

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

        is_required_layer_support_available(&vk_lib)
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
    // TODO populate
    score: u32
}

impl AppCapabilities {
    fn required_features(&self) -> DeviceFeatures {
        let all_features = get_required_device_features();
        all_features
    }

    fn required_extensions(&self) -> DeviceExtensions {
        let all_exts = get_required_device_extensions();
        all_exts
    }
    
    fn for_device_on_surface(physical_device: &Arc<PhysicalDevice>, surface: &Arc<Surface>) -> Option<Self> {
        let mut score = 0;

        score += match physical_device.clone().properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 5,
            PhysicalDeviceType::IntegratedGpu => 4,
            PhysicalDeviceType::VirtualGpu => 3,
            PhysicalDeviceType::Other => 2,
            PhysicalDeviceType::Cpu => 1,
            _ => { return None; }
        };

        let _caps = physical_device.surface_capabilities(&surface, Default::default()).ok()?;

        if physical_device.surface_formats(surface, Default::default()).ok()?.len() == 0 {
            return None;
        }

        is_required_device_support_available(physical_device).then_some(
            AppCapabilities {
                score
            })
    }

    fn score(&self) -> u32 { self.score }
}

struct AppResources {
    window: Arc<Window>,
    vulkan_surface: Arc<Surface>,
    capabilities: AppCapabilities,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>, // Graphics Q and Present Q may be the same
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
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

        let (physical_device, capabilities) = {
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
                    AppCapabilities::for_device_on_surface(&physical_device, &vulkan_surface)
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

        let graphics_queue_family_index = {
            let qfi_opt = physical_device
                .queue_family_properties()
                .iter()
                .position(|queue_family_properties| {
                    queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
                });

            match qfi_opt {
                Some(qfi) => qfi as u32,
                None => {
                    error!("Failed to find a suitable physical device!");
                    event_loop.exit();
                    return;
                }
            }
        };

        let present_queue_family_index = {
            let qfi_result = physical_device
                .queue_family_properties()
                .iter()
                .enumerate()
                .find_map(|(idx, queue_family_properties)| {
                    let idx32 = idx as u32;
                    match physical_device.presentation_support(idx32, event_loop) {
                        Ok(true) => Some(Ok(idx as u32)),
                        Ok(false) => None,
                        Err(e) => Some(Err(e))
                    }
                });

            match qfi_result {
                Some(Ok(qfi)) => qfi,
                Some(Err(e)) => {
                    error!("Failure while finding a physical device with present support! {e}");
                    event_loop.exit();
                    return;
                }
                None => {
                    error!("Failed to find a physical device with present support!");
                    event_loop.exit();
                    return;
                }
            }
        };

        let mut queue_create_info = vec![QueueCreateInfo {
            queue_family_index: graphics_queue_family_index,
            ..QueueCreateInfo::default()
        }];

        if graphics_queue_family_index != present_queue_family_index {
            queue_create_info.push(
                QueueCreateInfo {
                    queue_family_index: present_queue_family_index,
                    ..QueueCreateInfo::default()
                }
            )
        }

        let (device, mut queues) = match Device::new(physical_device, DeviceCreateInfo {
            enabled_features: capabilities.required_features(),
            enabled_extensions: capabilities.required_extensions(),
            queue_create_infos: queue_create_info,
            ..DeviceCreateInfo::default()
        }) {
            Ok(device) => device,
            Err(e) => {
                error!("Failed to create a logical device! {e}");
                event_loop.exit();
                return;
            }
        };

        info!("Using device {}", device.physical_device().properties().device_name);

        let graphics_queue = queues.next().unwrap();
        let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());

        // TODO Create Swapchain
        let surface_capabilities =
            match device.physical_device().surface_capabilities(&vulkan_surface, Default::default()) {
                Ok(caps) => caps,
                Err(e) => {
                    error!("Failed to get surface capabilities! {e}");
                    event_loop.exit();
                    return;
                }
            };

        let no_images =  min(max(surface_capabilities.min_image_count, 3), surface_capabilities.max_image_count.unwrap_or(u32::MAX));
        let composite_alpha = surface_capabilities.supported_composite_alpha.into_iter().next().unwrap();
        let image_format =  {
            let sfmts_result = device.physical_device()
                .surface_formats(&vulkan_surface, Default::default());
            
            match sfmts_result {
                Ok(sfmts) => {
                    debug!("Available image formats:\n{}", sfmts.iter().map(|(format, colorspace)| format!(" - {format:?}|{colorspace:?}", )).collect::<Vec<_>>().join("\n"));

                    sfmts.into_iter().next().expect("This should already have been checked")
                },
                Err(e) => {
                    error!("Failed to get surface formats! {e}");
                    event_loop.exit();
                    return;
                }
            }
        };

        debug!("Using {} images in Swapchain", no_images);

        let (swapchain, images) = {
            let swp_result = Swapchain::new(
                device.clone(),
                vulkan_surface.clone(),
                SwapchainCreateInfo {
                    min_image_count: no_images,
                    image_format: image_format.0,
                    image_extent: window.inner_size().into(),
                    image_usage: ImageUsage::COLOR_ATTACHMENT,
                    composite_alpha,
                    ..SwapchainCreateInfo::default()
                }
            );

            match swp_result {
                Ok(swp) => swp,
                Err(e) => {
                    error!("Failed to create swapchain! {e}");
                    event_loop.exit();
                    return;
                }
            }
        };

        self.resources = Some(AppResources {
            vulkan_surface,
            window,
            device,
            capabilities,
            graphics_queue,
            present_queue,
            swapchain,
            images
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
