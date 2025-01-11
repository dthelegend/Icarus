use crate::app::config::Config;
use crate::app::game::GameHandler;
use crate::app::resources::{RenderResources, ResourceError};
use crate::app::settings::Settings;
use crate::app::GameError;
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::thread::JoinHandle;
use thiserror::Error;
use vulkano::render_pass::RenderPass;
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
    #[error("Game error! {0}")]
    GameError(#[from] GameError),
    #[error("Invariant violation! No active resources!")]
    NoActiveResources
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

    pub fn run_game<Game: GameHandler>(self, game: &mut Game) -> Result<(), AppError> {
        game.on_start();
        
        let mut handler = AppHandler {
            app_name: self.app_name,
            render_resources: self.render_resources,
            settings: self.settings,
            game
        };

        self.event_loop.run_app(&mut handler)?;

        Ok(())
    }
}


struct AppHandler<'a, Game: GameHandler> {
    app_name: String,
    render_resources: RenderResources,
    settings: Settings,
    game: &'a mut Game
}

impl <T: GameHandler> AppHandler<'_, T> {
    
    fn draw(&mut self) -> Result<(), AppError> {
        // Let there be fish in the sea of love
        self.game.draw(&mut self.render_resources)?;

        Ok(())
    }
}

impl <T: GameHandler> ApplicationHandler for AppHandler<'_, T> {
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

        if let Err(e) = self.render_resources.create_device_resources(window.clone()) {
            error!("Failed to recreate Active Resources! {e}");
            event_loop.exit();
            return;
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
                    error!("Failed to draw on redraw request: {}", e);
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(_) => {
                if let Err(e) = &mut self.render_resources.recreate_swapchain() {
                    error!("Failed to recreate swapchain! {e}");
                    event_loop.exit();
                    return;
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = self.draw() {
            error!("Failed to draw when waiting: {}", e);
            event_loop.exit();
        }
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        self.render_resources.destroy_device_resources();
        debug!("App resources nuked!");
    }
}