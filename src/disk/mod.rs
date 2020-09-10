pub mod raw;
pub mod vhd;

use crate::{Error, Result};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiskFormat {
    RAW,
    VHD,
    // TODO:
    // VDI,
    // VMDK,
}

impl FromStr for DiskFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "raw" => Ok(Self::RAW),
            "vhd" => Ok(Self::VHD),
            _ => Err(Error::UnknownDiskType),
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

pub trait Info {
    fn disk_format(&self) -> DiskFormat;
    fn max_disk_size(&self) -> u64;
    fn disk_size(&self) -> u64;
    fn block_size(&self) -> u32;
    fn media_type(&self) -> MediaType;
}

pub trait Disk: io::Read + io::Seek + io::Write + Info {}

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

impl Default for ArgumentMap {
    fn default() -> Self {
        Self {
            0: Default::default(),
        }
    }
}

pub fn open_disk(
    format: DiskFormat,
    backend: Box<dyn Backend>,
    args: ArgumentMap,
) -> Result<Box<dyn Disk>> {
    Ok(match format {
        DiskFormat::RAW => Box::new(raw::RawDisk::open_with_argmap(backend, &args)),
        DiskFormat::VHD => Box::new(vhd::VhdDisk::open_with_argmap(backend, &args)?),
    })
}

pub struct Region<'a> {
    parent: &'a mut dyn Disk,
    start: u64,
    end: u64,
    cursor: u64,
}

impl<'a> Region<'a> {
    pub fn new(parent: &'a mut dyn Disk, start_lba: u64, end_lba: u64) -> Self {
        let disk_size = parent.max_disk_size();
        let sector_size = parent.block_size();

        let start = start_lba * sector_size as u64;
        let end = end_lba * sector_size as u64;
        let region_size = (end_lba - start_lba + 1) * sector_size as u64;

        assert!(start + region_size <= disk_size);

        Self {
            parent,
            start,
            end,
            cursor: 0,
        }
    }
}

impl<'a> io::Read for Region<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let s = self.start + self.cursor;
        if s > self.end {
            return Ok(0);
        }

        let l = buf.len() as u64;
        let mut e = s + l - 1;
        if e > self.end {
            e = self.end;
        }
        let l = e - s + 1;

        self.parent.seek(io::SeekFrom::Start(s))?;
        let r = self.parent.read(&mut buf[..l.try_into().unwrap()])?;
        self.cursor += r as u64;

        Ok(r)
    }
}

impl<'a> io::Seek for Region<'a> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            io::SeekFrom::Start(x) => self.cursor = x,
            //io::SeekFrom::End(x) => self.cursor = self.end + x,
            io::SeekFrom::Current(x) => self.cursor = self.cursor.wrapping_add(x as u64),
            io::SeekFrom::End(x) => self.cursor = self.max_disk_size() - x as u64,
        }

        Ok(self.cursor)
    }
}

impl<'a> io::Write for Region<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = self.start + self.cursor;
        if s > self.end {
            return Ok(0);
        }

        let l = buf.len() as u64;
        let mut e = s + l - 1;
        if e > self.end {
            e = self.end;
        }
        let l = e - s + 1;

        self.parent.seek(io::SeekFrom::Start(s))?;
        let w = self.parent.write(&buf[..l.try_into().unwrap()])?;
        self.cursor += w as u64;

        Ok(w)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.parent.flush()
    }
}

impl<'a> Disk for Region<'a> {}

impl<'a> Info for Region<'a> {
    fn disk_format(&self) -> DiskFormat {
        self.parent.disk_format()
    }
    fn max_disk_size(&self) -> u64 {
        self.end - self.start + 1
    }
    fn disk_size(&self) -> u64 {
        todo!()
    }
    fn block_size(&self) -> u32 {
        self.parent.block_size()
    }
    fn media_type(&self) -> MediaType {
        self.parent.media_type()
    }
}
