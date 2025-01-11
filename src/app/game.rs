use crate::app::resources;
use crate::app::resources::RenderResources;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error(transparent)]
    ResourceError(#[from] resources::ResourceError),
}

pub trait GameHandler {
    fn on_start(&mut self);
    fn draw(&mut self, resources: &mut RenderResources) -> Result<(), GameError>;
}