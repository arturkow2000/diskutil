use std::string::ToString;
use std::{io, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InvalidVhdFooter(Option<String>),
    InvalidVhdDynamicHeader(Option<String>),
    MbrMissing,
    GptMissing,
    InvalidGpt(String),
    UnknownDiskType,
    InvalidBpb,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Self::IoError(e) => e.to_string(),
            Self::InvalidVhdFooter(e) => e
                .as_ref()
                .map_or_else(|| "Invalid VHD Footer".to_owned(), |x| x.to_string()),
            Self::InvalidVhdDynamicHeader(e) => e.as_ref().map_or_else(
                || "Invalid VHD Dynamic Header".to_owned(),
                |x| x.to_string(),
            ),
            Self::MbrMissing => "MBR is missing".to_owned(),
            Self::GptMissing => "GPT is missing".to_owned(),
            Self::InvalidGpt(e) => e.clone(),
            Self::UnknownDiskType => "Unknown disk type".to_owned(),
            Self::InvalidBpb => "BPB is invalid".to_owned(),
        }
    }
}
