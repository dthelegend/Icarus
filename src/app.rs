use std::ffi::CStr;
use crate::consts::{API_VERSION, ENGINE_NAME, ENGINE_VERSION};
use ash::Entry;
use ash::vk;
use ash::vk::InstanceCreateFlags;
use std::ptr;
use log::{error, info};
use thiserror::Error;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{DisplayHandle, HandleError, HasDisplayHandle};
use winit::window::{Window, WindowId};

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    WindowEventError(#[from] winit::error::EventLoopError),
}

pub struct AppManager {
    _entry: Entry,
    app_instance: Option<AppInstance>,
    app_config: AppConfig,
}

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

impl AppManager {
    pub fn from_config(app_config: AppConfig) -> AppManager {
        let ash_entry = Entry::linked();

        AppManager {
            _entry: ash_entry,
            app_instance: None,
            app_config,
        }
    }
}

impl Default for AppManager {
    fn default() -> AppManager {
        Self::from_config(AppConfig::default())
    }
}

pub struct AppInstance {
    window: Window,
    vulkan_instance: ash::Instance,
}

impl ApplicationHandler for AppManager {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = match event_loop.create_window(Window::default_attributes().with_title("TODO Replace with title")) {
            Ok(window) => window,
            Err(e) => {
                info!("Failed to create Window! {e}");
                return;
            }
        };

        let app_name = c"TODO Replace with title";

        let app_info = vk::ApplicationInfo::default()
            .engine_name(ENGINE_NAME)
            .engine_version(ENGINE_VERSION)
            .api_version(API_VERSION)
            .application_name(app_name)
            .application_version(0);

        let surface_extensions = match event_loop.display_handle() {
            Ok(display_handle) =>
                match ash_window::enumerate_required_extensions(display_handle.as_raw()) {
                    Ok(extensions) => extensions,
                    Err(e) => {
                        error!("Failed to query Window extensions! {e}");
                        return;
                    }
                }
            Err(handle_error) => {
                error!("Failed to acquire display handle! {handle_error}");
                return;
            }
        };

        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(surface_extensions);

        let vulkan_instance = match unsafe { self._entry.create_instance(&create_info, None) } {
            Ok(vulkan_instance) => vulkan_instance,
            Err(e) => {
                error!("Failed to create Vulkan instance! {e}");
                return;
            }
        };

        self.app_instance = Some(AppInstance {
            window,
            vulkan_instance,
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
                self.app_instance.as_ref().unwrap().window.request_redraw();
            }
            _ => (),
        }
    }
}
