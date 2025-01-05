use std::sync::Arc;
use log::{debug, error, info, warn};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use thiserror::Error;
use vulkano::Version;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::window::{Window, WindowId};
use crate::app::config::Config;
use crate::app::resources::{ResourceError, StaticRenderResources};
use crate::app::settings::Settings;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("window event loop error! {0}")]
    WindowEventError(#[from] EventLoopError),
    #[error(transparent)]
    ResourceError(#[from] ResourceError),
}

// App manager produces instances
pub struct AppManager {
    event_loop: EventLoop<()>,
    render_resources: StaticRenderResources,
    settings: Settings
}

impl AppManager {
    pub fn from_config(config: Config) -> Result<Self, AppError> {
        let event_loop = EventLoop::new()?;
        
        let render_resources = StaticRenderResources::create(&event_loop, Some(config.app_name), Version::default())?;

        Ok(Self {
            event_loop,
            render_resources,
            settings: config.settings
        })
    }

    pub fn run(self) -> Result<(), AppError> {
        let mut handler = AppHandler {
            render_resources: self.render_resources,
            settings: self.settings
        };

        self.event_loop.run_app(&mut handler)?;

        Ok(())
    }
}


struct AppHandler {
    render_resources: StaticRenderResources,
    settings: Settings
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("TODO Replace with title")
            .with_inner_size(LogicalSize::new(self.settings.window_size[0], self.settings.window_size[1]));
        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                error!("Failed to create Window! {e}");
                event_loop.exit();
                return;
            }
        };

        debug!("Created a new window!");
        
        if let Err(e) = self.render_resources.recreate_active_resources(&window) {
            error!("Failed to recreate Active Resources! {e}");
        } else {
            debug!("Successfully created application resources!");
        }
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
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // if let Some(resources) = &mut self.resources {
        //     self.transient_resources = self.drawable.draw(resources, self.transient_resources.take());
        // } else {
        //     warn!("Application resources not available, but draw requested!");
        // }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.render_resources.destroy_active_resources();
        debug!("App resources nuked!");
    }
}
