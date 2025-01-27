use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("wgpu surface: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),

    #[error("failed to create wgpu surface: {0}")]
    CreateSurfaceError(#[from] wgpu::CreateSurfaceError),

    #[error("adapter request was None")]
    AdpaterRequestFailure,

    #[error("device request returned Error: {0}")]
    DeviceRequestFailure(#[from] wgpu::RequestDeviceError),

}

pub type Result<T> = core::result::Result<T, Error>;
