use crate::{connection::ConnectionError, frame::FrameError};
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("[Connection Error] {0}")]
    ConnectionError(#[from] ConnectionError),

    #[error("[Frame Error] {0}")]
    FrameError(#[from] FrameError),

    #[error("[IO Error] {0}")]
    IoError(#[from] io::Error),

    #[error("[Startup Error] {0}")]
    StartupError(StartupError),
}

#[derive(Error, Debug)]
#[error("{0}")]
pub struct StartupError(pub anyhow::Error);
