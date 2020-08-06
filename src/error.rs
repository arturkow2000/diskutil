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
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IoError(e)
    }
}
