use crate::app::config::Config;
use crate::app::resources::{ActiveRenderResources, RenderResources, ResourceError, TransientRenderResources};
use crate::app::settings::Settings;
use log::{debug, error, info, warn};
use std::sync::Arc;
use thiserror::Error;
use vulkano::Version;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("window event loop error! {0}")]
    WindowEventError(#[from] EventLoopError),
    #[error(transparent)]
    ResourceError(#[from] ResourceError),
    #[error("Draw error")]
    DrawError,
}

// App manager produces instances
pub struct AppManager {
    app_name: String,
    event_loop: EventLoop<()>,
    render_resources: RenderResources,
    settings: Settings,
}

impl AppManager {
    pub fn from_config(config: Config) -> Result<Self, AppError> {
        let event_loop = EventLoop::new()?;

        let render_resources = RenderResources::create(&event_loop, Some(config.app_name.clone()), Version::default())?;

        Ok(Self {
            app_name: config.app_name,
            event_loop,
            render_resources,
            settings: config.settings,
        })
    }

    pub fn run(self) -> Result<(), AppError> {
        let mut handler = AppHandler {
            app_name: self.app_name,
            render_resources: self.render_resources,
            settings: self.settings,
        };

        self.event_loop.run_app(&mut handler)?;

        Ok(())
    }
}


struct AppHandler {
    app_name: String,
    render_resources: RenderResources,
    settings: Settings,
}

impl AppHandler {
    fn draw(&mut self) -> Result<(), AppError> {
        let active_resources = self.render_resources.active_resources.as_mut().ok_or(AppError::DrawError)?;

        let tmp_transient_render_resources = active_resources.transient_render_resources.take().ok_or(AppError::DrawError)?;

        // TODO Some drawing

        active_resources.transient_render_resources = Some(tmp_transient_render_resources);

        Ok(())
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(self.app_name.clone())
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

        self.render_resources.active_resources = match ActiveRenderResources::new(&self.render_resources, window.clone()) {
            Ok(active_resources) => {
                debug!("Successfully created application resources!");
                Some(active_resources)
            }
            Err(e) => {
                error!("Failed to recreate Active Resources! {e}");
                event_loop.exit();
                return;
            }
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
                if let Err(e) = self.draw() {
                    error!("Failed to draw: {}", e);
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(_) => {
                if let Some(active_resources) = &mut self.render_resources.active_resources {
                    active_resources.transient_render_resources = None;
                    if let Err(e) = self.draw() {
                        error!("Failed to draw: {}", e);
                        event_loop.exit();
                    }
                } else {
                    error!("Invariant violated: No active resources!");
                    event_loop.exit();
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = self.draw() {
            error!("Failed to draw: {}", e);
            event_loop.exit();
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.render_resources.active_resources = None;
        debug!("App resources nuked!");
    }
}