mod disk;
mod dynamic_header;
mod footer;

pub use disk::VhdDisk;

use crate::Error;
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u32)]
pub enum DiskType {
    Fixed = 2,
    Dynamic = 3,
    Differencing = 4,
}

impl TryFrom<u32> for DiskType {
    type Error = Error;

    fn try_from(x: u32) -> Result<Self, Self::Error> {
        match x {
            2 => Ok(Self::Fixed),
            3 => Ok(Self::Dynamic),
            4 => Ok(Self::Differencing),
            _ => Err(Error::InvalidVhdFooter(Some(
                "Invalid VHD disk type".to_owned(),
            ))),
        }
    }
}

/*impl Into<u32> for DiskType {
    fn into(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Fixed => 2,
            Self::Dynamic => 3,
            Self::Differencing => 4,
            Self::Unknown(x) => x,
        }
    }
}
*/
