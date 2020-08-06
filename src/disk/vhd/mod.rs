mod disk;
mod dynamic_header;
mod footer;

pub use disk::VhdDisk;

#[derive(Debug, PartialEq)]
pub enum DiskType {
    Unknown(u32),
    None,
    Fixed,
    Dynamic,
    Differencing,
}

impl From<u32> for DiskType {
    fn from(x: u32) -> Self {
        match x {
            0 => Self::None,
            2 => Self::Fixed,
            3 => Self::Dynamic,
            4 => Self::Differencing,
            x => Self::Unknown(x),
        }
    }
}

impl Into<u32> for DiskType {
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
