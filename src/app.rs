use std::ffi::CStr;
use crate::consts::{API_VERSION, ENGINE_NAME, ENGINE_VERSION};
use ash::vk;
use ash::vk::InstanceCreateFlags;
use std::ptr;
use ash::prelude::VkResult;
use log::{debug, error, info};
use thiserror::Error;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle};
use winit::window::{Window, WindowId};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("window event loop error")]
    WindowEventError(#[from] winit::error::EventLoopError),
    #[error("failed to acquire raw window handle")]
    HandleError(#[from] HandleError),
    #[error("vulkan error ({0})")]
    VulkanError(#[from] vk::Result),
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
pub struct AppManager<'a> {
    _entry: ash::Entry,
    app_config: AppConfig,
    event_loop: EventLoop<()>,
    vulkan_instance: ash::Instance,
    display_handle: DisplayHandle<'a>
}

impl <'a> AppManager<'a> {
    pub fn from_config(app_config: AppConfig) -> Result<AppManager<'a>, AppError> {
        let event_loop = EventLoop::new()?;
        let ash_entry = ash::Entry::linked();

        let app_info = vk::ApplicationInfo::default()
            .engine_name(ENGINE_NAME)
            .engine_version(ENGINE_VERSION)
            .api_version(API_VERSION)
            .application_name(app_config.app_name)
            .application_version(0);

        let display_handle = event_loop.display_handle()?;
        let surface_extensions = ash_window::enumerate_required_extensions(display_handle.as_raw())?;

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(surface_extensions);

        let vulkan_instance = unsafe { ash_entry.create_instance(&create_info, None) }?;

        Ok(AppManager {
            _entry: ash_entry,
            app_config,
            event_loop,
            vulkan_instance,
            display_handle
        })
    }

    pub fn run(self) -> Result<(), AppError> {
        let mut handler = AppHandler {
            _entry: self._entry,
            vulkan_instance: self.vulkan_instance,
            display_handle: self.display_handle,
            resources: None
        };

        self.event_loop.run_app(&mut handler).map_err(From::from)
    }
}

impl <'a> Drop for AppManager<'a> {
    fn drop(&mut self) {
        unsafe { self.vulkan_instance.destroy_instance(None); }
    }
}

struct AppResources {
    window: Window,
    surface: vk::SurfaceKHR,
}

struct AppHandler<'a> {
    _entry: ash::Entry,
    vulkan_instance: ash::Instance,
    display_handle: DisplayHandle<'a>,
    resources: Option<AppResources>,
}

impl <'a> ApplicationHandler for AppHandler<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = match event_loop.create_window(Window::default_attributes().with_title("TODO Replace with title")) {
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

        let surface_result = unsafe {
            ash_window::create_surface(
                &self._entry,
                &self.vulkan_instance,
                self.display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
        };

        let surface = match surface_result {
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
        let resources = None;
        (self.resources, resources) = (resources, self.resources);
        match resources {
            Some(app_resources) => {
                unsafe {
                    self.vulkan_instance.destroy_instance(app_resources.surface, None)
                }
            }
            None => {}
        }
    }
}
