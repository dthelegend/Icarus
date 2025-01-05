use crate::app::capabilities::{Capabilities, CapabilityError};
use std::sync::Arc;
use thiserror::Error;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::image::{Image, ImageAspects, ImageSubresourceRange, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use vulkano::{LoadingError, Validated, Version, VulkanError, VulkanLibrary};
use vulkano::instance::debug::{DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessengerCreateInfo};
use winit::event_loop::EventLoop;
use winit::raw_window_handle::HandleError;
use winit::window::Window;
use crate::app::resources::utils::{get_debug_utils_callback, get_required_layers, is_required_layer_support_available, REQUIRED_INSTANCE_EXTENSIONS};

mod utils;

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("failed to load Vulkan! {0}")]
    LoadingError(#[from] LoadingError),
    #[error("failed to acquire raw window handle! {0}")]
    HandleError(#[from] HandleError),
    #[error("unable to find required layers")]
    VulkanMissingLayers,
    #[error("unable to find a suitable device")]
    VulkanNoSuitableDevice,
    #[error("vulkan error! {0}")]
    VulkanError(#[from] VulkanError),
    #[error("vulkan error! {0}")]
    ValidatedVulkanError(#[from] Validated<VulkanError>)
}

/// Resources that may be destroyed an
pub struct TransientRenderResources {
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    frame_buffers: Vec<Arc<Framebuffer>>,
}

/// Resources that should be destroyed and recreated alongside the window
pub struct ActiveRenderResources {
    vulkan_surface: Arc<Surface>,
    capabilities: Capabilities,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>, // Graphics Q and Present Q may be the same
    render_size: [u32; 2],

    // Ensures our transient resources cannot live longer than our static ones
    transient_render_resources: Option<TransientRenderResources>
}

impl ActiveRenderResources {
    pub fn recreate_transient_resources(&mut self, render_pass: &Arc<RenderPass>) -> Result<(), ResourceError> {
        let (swapchain, images) = Swapchain::new(
            self.device.clone(),
            self.vulkan_surface.clone(),
            SwapchainCreateInfo {
                min_image_count: self.capabilities.swapchain_images(),
                image_format: self.capabilities.image_format().0,
                image_extent: self.render_size,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: *self.capabilities.composite_alpha(),
                ..SwapchainCreateInfo::default()
            }
        )?;

        let frame_buffers = images
            .iter()
            .cloned()
            .map(|image| {
                let create_info = ImageViewCreateInfo {
                    view_type: ImageViewType::Dim2d,
                    subresource_range: ImageSubresourceRange {
                        aspects: ImageAspects::COLOR,
                        array_layers: 0..1,
                        mip_levels: 0..1
                    },
                    ..ImageViewCreateInfo::from_image(&image)
                };

                let view = ImageView::new(image, create_info)?;

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..FramebufferCreateInfo::default()
                    }
                )
            }).collect::<Result<_,_>>()?;
        
        self.transient_render_resources = Some(TransientRenderResources {
            swapchain,
            images,
            frame_buffers,
        });
        
        Ok(())
    }
    
    pub fn destroy_transient_resources(&mut self) {
        self.transient_render_resources = None;
    }
}

/// Resources that live as long as the application
pub struct StaticRenderResources {
    vulkan_instance: Arc<Instance>,
    
    // Ensures our active resources cannot live longer than our static ones
    active_render_resources: Option<ActiveRenderResources>
}

impl StaticRenderResources {
    pub fn create(event_loop: &EventLoop<()>, application_name: Option<String>, application_version: Version) -> Result<Self, ResourceError> {
        let vk_lib = VulkanLibrary::new()?;

        is_required_layer_support_available(&vk_lib)
            .map_err(From::from)
            .and_then(|is_supported| is_supported.then_some(()).ok_or(ResourceError::VulkanMissingLayers))?;

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
            enabled_extensions: Surface::required_extensions(&event_loop)?.union(&REQUIRED_INSTANCE_EXTENSIONS),
            enabled_layers: get_required_layers(),
            debug_utils_messengers,
            application_name,
            application_version,
            ..InstanceCreateInfo::application_from_cargo_toml()
        })?;

        Ok(StaticRenderResources {
            vulkan_instance,
            active_render_resources: None,
        })
    }
    
    /// initialises some active resources; will overwrite existing resources
    pub fn recreate_active_resources(&mut self, window: &Arc<Window>) -> Result<(), ResourceError> {
        let vulkan_surface = Surface::from_window(self.vulkan_instance.clone(), window)?;

        let (physical_device, capabilities) = self.vulkan_instance.enumerate_physical_devices()?
            .map(|physical_device| {
                let caps = Capabilities::for_device_on_surface(&physical_device, &vulkan_surface);

                (physical_device, caps)
            })
            .filter_map(|(pd, cap_result)| match cap_result {
                Ok(cap) => {
                    Some(Ok((pd, cap)))
                }
                Err(CapabilityError::Unsuitable) => {
                    None
                }
                Err(CapabilityError::VulkanError(vk_error)) => {
                    Some(Err(vk_error))
                }
            })
            .collect::<Result<_,_>>()?
            .max_by_key(|(_physical_device, d)| d.score())
            .ok_or(ResourceError::VulkanNoSuitableDevice)?;

        let graphics_queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .position(|queue_family_properties| {
                queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
            })
            .ok_or(ResourceError::VulkanNoSuitableDevice)?;

        let present_queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find_map(|(idx, queue_family_properties)| {
                let idx32 = idx as u32;
                match physical_device.presentation_support(idx32, window) {
                    Ok(true) => Some(Ok(idx32)),
                    Ok(false) => None,
                    Err(e) => Some(Err(e))
                }
            });

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

        let (device, mut queues) = Device::new(physical_device, DeviceCreateInfo {
            enabled_features: capabilities.required_features(),
            enabled_extensions: capabilities.required_extensions(),
            queue_create_infos: queue_create_info,
            ..DeviceCreateInfo::default()
        })?;
        
        let graphics_queue = queues.next().unwrap();
        let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());
        
        self.active_render_resources = Some(ActiveRenderResources {
            vulkan_surface,
            capabilities,
            device,
            graphics_queue,
            present_queue,
            render_size: window.inner_size().into(),
            transient_render_resources: None,
        });
        
        Ok(())
    }
    
    /// destroys the active resources
    pub fn destroy_active_resources(&mut self) {
        self.active_render_resources = None;
    }
}
