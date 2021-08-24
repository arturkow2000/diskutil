use std::{io, result};

use thiserror::Error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("invalid VHD footer {0:#?}")]
    InvalidVhdFooter(Option<String>),
    #[error("invalid VHD dynamic header {0:#?}")]
    InvalidVhdDynamicHeader(Option<String>),
    #[error("MBR is missing")]
    MbrMissing,
    #[error("GPT is missing")]
    GptMissing,
    #[error("{0}")]
    InvalidGpt(String),
    #[error("unknown disk type")]
    UnknownDiskType,
    #[error("invalid BPB")]
    InvalidBpb,
    #[error("not supported")]
    NotSupported,
    #[error("not found")]
    NotFound,
}
