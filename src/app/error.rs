use thiserror::Error;

use super::render;

#[derive(Error, Debug)]
pub enum Error {

    #[error("failure creating window")]
    CreateWindow(#[from] winit::error::OsError),

    #[error("renderer failure: {0}")]
    Renderer(#[from] render::error::Error),

}

pub type Result<T> = core::result::Result<T, Error>;
