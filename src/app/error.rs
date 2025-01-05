use thiserror::Error;
use vulkano::{LoadingError, Validated, VulkanError};
use winit::error::EventLoopError;
use winit::raw_window_handle::HandleError;

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