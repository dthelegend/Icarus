use thiserror::Error;
use crate::app::resources;
use crate::app::resources::ActiveRenderResources;

#[derive(Error, Debug)]
pub enum GameError {
    #[error(transparent)]
    ResourceError(#[from] resources::ResourceError),
}

pub trait GameHandler {
    fn on_start(&mut self);
    fn draw(&mut self, resources: &mut ActiveRenderResources) -> Result<(), GameError>;
}