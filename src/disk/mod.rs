use std::cmp::min;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::fs::File;
use std::io;
use std::str::FromStr;

use crate::{Error, Result};
pub use slice::DiskSlice;

pub mod buffer;
pub mod ram;
pub mod raw;
mod slice;
pub mod vhd;

#[cfg(all(feature = "device", windows))]
pub use windows_device::DeviceBackend;

#[cfg(all(feature = "device", windows))]
mod windows_device;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiskFormat {
    Device,
    RAW,
    VHD,
    // TODO:
    // VHDX
    // VDI,
    // VMDK,
}

impl FromStr for DiskFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "device" => Ok(Self::Device),
            "raw" => Ok(Self::RAW),
            "vhd" => Ok(Self::VHD),
            _ => Err(Error::UnknownDiskType),
        }
    }
}

impl fmt::Display for DiskFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Device => write!(f, "device"),
            Self::RAW => write!(f, "raw"),
            Self::VHD => write!(f, "vhd"),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum MediaType {
    Unknown,
    FDD,
    HDD,
    SSD,
    CDROM,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WipePolarity {
    DontCare,
    Low,
    High,
}

pub trait Disk: io::Read + io::Seek + io::Write {
    /// Returns disk size in bytes
    fn disk_size(&self) -> u64;

    /// Returns block size device can read or write
    fn sector_size(&self) -> u32;
    fn media_type(&self) -> MediaType;
    fn disk_format(&self) -> DiskFormat;

    /// Fills specified disk region with either zero's or one's.
    fn wipe(&mut self, size: usize, polarity: WipePolarity) -> Result<()> {
        // TODO: flash memories support erase operation,
        // maybe we could take advantage of that to increase write speed

        // Allocate at most 16 MiB
        const BLOCK_SIZE: usize = 16777216;
        let mut left = size;

        let mut buf: Vec<u8> = Vec::with_capacity(min(BLOCK_SIZE, left));
        buf.resize(
            min(BLOCK_SIZE, left),
            match polarity {
                WipePolarity::DontCare | WipePolarity::Low => 0,
                WipePolarity::High => 0xff,
            },
        );

        while left > 0 {
            let n = min(left, 16777216);
            self.write_all(&buf[..n])?;
            left -= n;
        }

        Ok(())
    }
}

pub trait Backend: io::Read + io::Seek + io::Write {
    fn data_length(&self) -> u64;
}
pub struct FileBackend {
    file: File,
    data_length: u64,
}
impl FileBackend {
    pub fn new(file: File) -> Result<Box<Self>> {
        let m = file.metadata()?;

        Ok(Box::new(Self {
            file,
            data_length: m.len(),
        }))
    }
    pub fn into_inner(self) -> File {
        self.file
    }
}

impl io::Read for FileBackend {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.file.read_vectored(bufs)
    }
}

impl io::Seek for FileBackend {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

impl io::Write for FileBackend {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.file.write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Backend for FileBackend {
    fn data_length(&self) -> u64 {
        self.data_length
    }
}

pub enum Argument {
    Signed(i64),
    Unsigned(u64),
    String(String),
}

#[derive(Default)]
pub struct ArgumentMap(HashMap<String, Argument>);
macro_rules! g1 {
    ($name:tt, $type:ty) => {
        pub fn $name(&self, key: &str) -> Option<$type> {
            self.0.get_key_value(key).and_then(|x| match x.1 {
                Argument::Signed(x) => {
                    if let Ok(x) = TryInto::<$type>::try_into(*x) {
                        Some(x)
                    } else {
                        None
                    }
                }
                Argument::Unsigned(x) => {
                    if let Ok(x) = TryInto::<$type>::try_into(*x) {
                        Some(x)
                    } else {
                        None
                    }
                }
                Argument::String(_) => None,
            })
        }
    };
}
impl ArgumentMap {
    g1!(get_i8, i8);
    g1!(get_i16, i16);
    g1!(get_i32, i32);
    g1!(get_i64, i64);

    g1!(get_u8, u8);
    g1!(get_u16, u16);
    g1!(get_u32, u32);
    g1!(get_u64, u64);
}

pub fn open_disk(
    format: DiskFormat,
    backend: Box<dyn Backend>,
    args: ArgumentMap,
) -> Result<Box<dyn Disk>> {
    Ok(match format {
        DiskFormat::Device => {
            let disk = raw::RawDisk::open_with_argmap(backend, &args);
            let buffer = buffer::Buffer::new(disk)?;
            Box::new(buffer)
        }
        DiskFormat::RAW => Box::new(raw::RawDisk::open_with_argmap(backend, &args)),
        DiskFormat::VHD => Box::new(vhd::VhdDisk::open_with_argmap(backend, &args)?),
    })
}
