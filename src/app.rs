use std::ffi::CStr;
use crate::consts::{API_VERSION, ENGINE_NAME, ENGINE_VERSION};
use ash::vk;
use ash::vk::InstanceCreateFlags;
use std::ptr;
use ash::prelude::VkResult;
use log::{debug, error, info, log_enabled};
use log::Level::Debug;
use thiserror::Error;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle};
use winit::window::{Window, WindowId};
use crate::vulkan::{VulkanError, VulkanInstance};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("window event loop error")]
    WindowEventError(#[from] winit::error::EventLoopError),
    #[error("failed to acquire raw window handle")]
    HandleError(#[from] HandleError),
    #[error(transparent)]
    VulkanError(#[from] VulkanError),
}

// Config
// TODO make this constructible using a builder
pub struct AppConfig {
    app_name: &'static CStr
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            app_name: c"Icarus Engine"
        }
    }
}

// App manager produces instances
pub struct AppManager {
    app_config: AppConfig,
    event_loop: EventLoop<()>,
    vulkan_instance: VulkanInstance
}

impl AppManager {
    pub fn from_config(app_config: AppConfig) -> Result<AppManager, AppError> {
        let event_loop = EventLoop::new()?;

        let app_info = vk::ApplicationInfo::default()
            .engine_name(ENGINE_NAME)
            .engine_version(ENGINE_VERSION)
            .api_version(API_VERSION)
            .application_name(app_config.app_name)
            .application_version(0);

        let display_handle = event_loop.display_handle()?;

        let vulkan_instance = VulkanInstance::create(&display_handle, app_info)?;

        Ok(AppManager {
            app_config,
            event_loop,
            vulkan_instance
        })
    }

    pub fn run(self) -> Result<(), AppError> {
        let mut handler = AppHandler {
            vulkan_instance: self.vulkan_instance,
            resources: None
        };

        self.event_loop.run_app(&mut handler).map_err(From::from)
    }
}

struct AppResources {
    window: Window,
    surface: vk::SurfaceKHR,
}

struct AppHandler {
    vulkan_instance: VulkanInstance,
    resources: Option<AppResources>,
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("TODO Replace with title")
            .with_inner_size(LogicalSize::new(1920, 1080));
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => window,
            Err(e) => {
                error!("Failed to create Window! {e}");
                return;
            }
        };

        debug!("Created a new window ({:?})", window.id());

        let window_handle = match window.window_handle() {
            Ok(window_handle) => window_handle,
            Err(e) => {
                error!("Failed to get window handle! {e}");
                return;
            }
        };

        let display_handle = match event_loop.display_handle() {
            Ok(display_handle) => display_handle,
            Err(e) => {
                error!("Failed to get display handle! {e}");
                return;
            }
        };

        let surface = match self.vulkan_instance.create_surface(&display_handle, &window_handle) {
            Ok(surface) => surface,
            Err(e) => {
                error!("Failed to create a window surface! {e}");
                return;
            }
        };

        self.resources = Some(AppResources {
            window,
            surface,
        });
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
        let mut resources = self.resources.take();
        if let Some(app_resources) = resources {
            // TODO this surface is not correctly destroyed
            // unsafe {
            //     self.vulkan_instance.destroy_instance(app_resources.surface, None)
            // }
        }
    }
}
