use thiserror::Error;

use crate::app;


#[derive(Error, Debug)]
pub enum Error {
    #[error("config error: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("winit error: {0}")]
    WinitError(#[from] winit::error::EventLoopError),

    #[error("application error: {0}")]
    ApplicationError(#[from] app::error::Error),

}

pub type Result<T> = core::result::Result<T, Error>;
