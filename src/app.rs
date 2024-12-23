use std::ptr;
use ash::Entry;
use ash::vk;
use ash::vk::InstanceCreateFlags;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};
use thiserror::Error;
use winit::raw_window_handle::HasDisplayHandle;
use crate::consts::{API_VERSION, ENGINE_NAME, ENGINE_VERSION};

#[derive(Error)]
enum AppError {
    WindowEventError(#[from] winit::error::EventLoopError),
}

pub struct AppManager {
    _entry: Entry,
    app_instance: Option<AppInstance>,
}

impl AppManager {
    pub fn new() -> AppManager {
        let ash_entry = Entry::linked();

        AppManager {
            _entry: ash_entry,
            app_instance: None,
        }
    }
}

pub struct AppInstance {
    window: Window,
    vulkan_instance: ash::Instance,
}

impl ApplicationHandler for AppManager {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Ok(window) = event_loop.create_window(
            Window::default_attributes()
                .with_title("TODO Replace with title")
        ) {
            let app_name = c"TODO Replace with title";

            let app_info = vk::ApplicationInfo::default()
                .engine_name(ENGINE_NAME)
                .engine_version(ENGINE_VERSION)
                .api_version(API_VERSION)
                .application_name(app_name)
                .application_version(0);

            // TODO handle correctly
            let surface_extensions =
                ash_window::enumerate_required_extensions(event_loop.display_handle().unwrap().as_raw()).unwrap();

            let create_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_extension_names(surface_extensions);

            let instance_result = unsafe {
                self._entry
                    .create_instance(&create_info, None)
            };

            if let Ok(vulkan_instance) = instance_result {
                self.app_instance = Some(AppInstance {
                    window,
                    vulkan_instance,
                })
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // pass
            }
            _ => (),
        }
    }
}

fn main_loop() -> Result<(), AppError> {
    let event_loop = EventLoop::new()?;

    let mut appman = AppManager::new();

    event_loop.run_app(&mut appman)?;

    Ok(())
}