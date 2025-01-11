use crate::app::capabilities::{Capabilities, CapabilityError};
use crate::app::resources::utils::{get_debug_utils_callback, get_required_layers, is_required_layer_support_available, REQUIRED_INSTANCE_EXTENSIONS};
use log::warn;
use std::sync::Arc;
use thiserror::Error;
use vulkano::buffer::Subbuffer;
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::device::{Device, DeviceCreateInfo, DeviceOwned, Queue, QueueCreateInfo, QueueFlags};
use vulkano::format::Format;
use vulkano::image::view::{ImageView, ImageViewCreateInfo, ImageViewType};
use vulkano::image::{Image, ImageAspects, ImageSubresourceRange, ImageUsage};
use vulkano::instance::debug::{DebugUtilsMessageSeverity, DebugUtilsMessageType, DebugUtilsMessengerCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::pipeline::graphics::vertex_input::VertexBuffersCollection;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{FromWindowError, Surface, Swapchain, SwapchainCreateInfo};
use vulkano::{LoadingError, Validated, ValidationError, Version, VulkanError, VulkanLibrary};
use winit::event_loop::EventLoop;
use winit::raw_window_handle::HandleError;
use winit::window::Window;

mod utils;

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("failed to load Vulkan! {0}")]
    LoadingError(#[from] LoadingError),
    #[error("failed to acquire raw window handle! {0}")]
    HandleError(#[from] HandleError),
    #[error("failed to acquire raw window handle! {0}")]
    ValidatedHandleError(#[from] Validated<HandleError>),
    #[error("unable to find required layers")]
    VulkanMissingLayers,
    #[error("unable to find a suitable device")]
    VulkanNoSuitableDevice,
    #[error("vulkan error! {0}")]
    VulkanError(#[from] VulkanError),
    #[error("vulkan error! {0}")]
    ValidatedVulkanError(#[from] Validated<VulkanError>),
    #[error("failed to create surface from window! {0}")]
    SurfaceCreationError(#[from] FromWindowError),
    #[error("failed to build graphics pipeline!")]
    GraphicsPipelineError(#[from] Box<ValidationError>)
}

/// Resources that may be destroyed any time
struct SwapchainResources {
    render_pass: Arc<RenderPass>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    frame_buffers: Vec<Arc<Framebuffer>>,
}

impl SwapchainResources {
    pub fn new(active_resources: &DeviceResources) -> Result<Self, ResourceError> {
        let (swapchain, images) = Swapchain::new(
            active_resources.device.clone(),
            active_resources.vulkan_surface.clone(),
            SwapchainCreateInfo {
                min_image_count: active_resources.capabilities.swapchain_images(),
                image_format: active_resources.capabilities.image_format().0,
                image_extent: active_resources.window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: *active_resources.capabilities.composite_alpha(),
                ..SwapchainCreateInfo::default()
            },
        )?;

        let render_pass = vulkano::single_pass_renderpass!(
            active_resources.device.clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
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
                        mip_levels: 0..1,
                    },
                    ..ImageViewCreateInfo::from_image(&image)
                };

                let view = ImageView::new(image, create_info)?;

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..FramebufferCreateInfo::default()
                    },
                )
            }).collect::<Result<_, _>>()?;

        Ok(SwapchainResources {
            render_pass,
            swapchain,
            images,
            frame_buffers,
        })
    }

    pub fn recreate(self, new_size: [u32; 2]) -> Result<Self, ResourceError> {
        let swapchain_recreate_info = SwapchainCreateInfo {
            image_extent: new_size,
            ..self.swapchain.create_info()
        };
        
        let (swapchain, images) = self.swapchain.recreate(swapchain_recreate_info)?;

        let render_pass = vulkano::single_pass_renderpass!(
            swapchain.device().clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
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
                        mip_levels: 0..1,
                    },
                    ..ImageViewCreateInfo::from_image(&image)
                };

                let view = ImageView::new(image, create_info)?;

                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..FramebufferCreateInfo::default()
                    },
                )
            }).collect::<Result<_, _>>()?;

        Ok(SwapchainResources {
            render_pass,
            swapchain,
            images,
            frame_buffers,
        })
    }
}

/// Resources that should be destroyed and recreated alongside the window
struct DeviceResources {
    window: Arc<Window>,
    vulkan_surface: Arc<Surface>,
    capabilities: Capabilities,
    device: Arc<Device>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>, // Graphics Q and Present Q may be the same,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,

    // Ensures our transient resources cannot live longer than our static ones
    swapchain_resources: Option<SwapchainResources>,
}

impl DeviceResources {
    pub fn new(render_resources: &RenderResources, window: Arc<Window>) -> Result<Self, ResourceError> {
        let vulkan_surface = Surface::from_window(render_resources.vulkan_instance.clone(), window.clone())?;

        let (physical_device, capabilities) = render_resources.vulkan_instance.enumerate_physical_devices()?
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
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max_by_key(|(_physical_device, d)| d.score())
            .ok_or(ResourceError::VulkanNoSuitableDevice)?;

        let graphics_queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .position(|queue_family_properties| {
                queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
            })
            .ok_or(ResourceError::VulkanNoSuitableDevice)? as u32;

        let present_queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find_map(|(idx, queue_family_properties)| {
                let idx32 = idx as u32;
                match physical_device.presentation_support(idx32, &window) {
                    Ok(true) => Some(Ok(idx32)),
                    Ok(false) => None,
                    Err(e) => Some(Err(e))
                }
            })
            .ok_or(ResourceError::VulkanNoSuitableDevice)??;

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
            enabled_features: *capabilities.required_features(),
            enabled_extensions: *capabilities.required_extensions(),
            queue_create_infos: queue_create_info,
            ..DeviceCreateInfo::default()
        })?;

        let graphics_queue = queues.next().unwrap();
        let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo {
                ..StandardCommandBufferAllocatorCreateInfo::default()
            }
        ));

        Ok(DeviceResources {
            window,
            vulkan_surface,
            capabilities,
            device,
            graphics_queue,
            present_queue,
            command_buffer_allocator,

            swapchain_resources: None,
        })
    }
}

/// Resources that live as long as the application
pub struct RenderResources {
    vulkan_instance: Arc<Instance>,

    // Ensures our active resources cannot live longer than our static ones
    device_resources: Option<DeviceResources>,
}

impl RenderResources {
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

        Ok(RenderResources {
            vulkan_instance,
            device_resources: None,
        })
    }

    pub fn destroy_swapchain(&mut self) -> Result<&mut Self, ResourceError> {
        if let Some(active_resources) = &mut self.device_resources {
            active_resources.swapchain_resources = None;
        } else {
            warn!("Destroying swapchain without active resources!");
        }

        Ok(self)
    }

    pub fn recreate_swapchain(&mut self) -> Result<&mut Self, ResourceError> {
        if let Some(device_resources) = &mut self.device_resources {
            let swapchain_resources = match device_resources.swapchain_resources.take() {
                Some(swapchain_resources) => swapchain_resources.recreate(device_resources.window.inner_size().into())?,
                None => SwapchainResources::new(device_resources)?
            };
            
            device_resources.swapchain_resources = Some(swapchain_resources);
        } else {
            warn!("Recreating swapchain without device resources!");
        }
        
        Ok(self)
    }

    pub fn create_device_resources(&mut self,  window: Arc<Window>) -> Result<&mut Self, ResourceError> {
        self.device_resources = Some(DeviceResources::new(self, window)?);
        
        Ok(self)
    }
    
    pub fn destroy_device_resources(&mut self) -> &mut Self {
        self.device_resources = None;

        self
    }
}
