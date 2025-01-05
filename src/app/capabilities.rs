use std::cmp::{max, min};
use std::sync::Arc;
use thiserror::Error;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{DeviceExtensions, DeviceFeatures};
use vulkano::format::Format;
use vulkano::swapchain::{ColorSpace, CompositeAlpha, Surface};
use vulkano::{Validated, VulkanError};

#[derive(Error, Debug)]
pub enum CapabilityError {
    #[error("vulkan error! {0}")]
    VulkanError(#[from] Validated<VulkanError>),
    #[error("GPU is unsuitable")]
    Unsuitable,
}


/// Capabilities describes everything that the GPU can do as well as a score to rank
pub struct Capabilities {
    device_features: DeviceFeatures,
    device_extensions: DeviceExtensions,
    score: u32,
    swapchain_images: u32,
    composite_alpha: CompositeAlpha,
    image_format: (Format, ColorSpace),
}

const REQUIRED_DEVICE_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    khr_swapchain: true,
    ..DeviceExtensions::empty()
};

const REQUIRED_DEVICE_FEATURES: DeviceFeatures = DeviceFeatures {
    ..DeviceFeatures::empty()
};

const OPTIONAL_DEVICE_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    ..REQUIRED_DEVICE_EXTENSIONS
};

const OPTIONAL_DEVICE_FEATURES: DeviceFeatures = DeviceFeatures {
    ..REQUIRED_DEVICE_FEATURES
};

impl Capabilities {
    pub fn for_device_on_surface(physical_device: &Arc<PhysicalDevice>, surface: &Arc<Surface>) -> Result<Self, CapabilityError> {
        let mut score = 0;

        score += match physical_device.clone().properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 5,
            PhysicalDeviceType::IntegratedGpu => 4,
            PhysicalDeviceType::VirtualGpu => 3,
            PhysicalDeviceType::Other => 2,
            PhysicalDeviceType::Cpu => 1,
            _ => 0
        };

        let caps = physical_device.surface_capabilities(&surface, Default::default())?;

        let swapchain_images = min(max(caps.min_image_count, 3), caps.max_image_count.unwrap_or(u32::MAX));
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = *physical_device.surface_formats(surface, Default::default())?.first().ok_or(CapabilityError::Unsuitable)?;

        if
        !physical_device.supported_features().contains(&REQUIRED_DEVICE_FEATURES)
            || !physical_device.supported_extensions().contains(&OPTIONAL_DEVICE_EXTENSIONS)
        {
            Err(CapabilityError::Unsuitable)
        } else {
            Ok(Capabilities {
                device_features: physical_device.supported_features().intersection(&OPTIONAL_DEVICE_FEATURES),
                device_extensions: physical_device.supported_extensions().intersection(&OPTIONAL_DEVICE_EXTENSIONS),
                swapchain_images,
                composite_alpha,
                image_format,
                score,
            })
        }
    }

    pub fn score(&self) -> u32 { self.score }

    pub fn required_features(&self) -> &DeviceFeatures { &self.device_features }
    pub fn required_extensions(&self) -> &DeviceExtensions { &self.device_extensions }

    pub fn swapchain_images(&self) -> u32 {
        self.swapchain_images
    }

    pub fn composite_alpha(&self) -> &CompositeAlpha {
        &self.composite_alpha
    }

    pub fn image_format(&self) -> &(Format, ColorSpace) {
        &self.image_format
    }
}