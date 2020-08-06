pub mod raw;
pub mod vhd;

use std::io;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum DiskType {
    Unknown,
    RAW,
    VHD,
    VDI,
    VMDK,
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
    fn disk_type(&self) -> DiskType;
    fn max_disk_size(&self) -> usize;
    fn disk_size(&self) -> usize;
    fn block_size(&self) -> usize;
    fn media_type(&self) -> MediaType;
}

pub trait Disk: io::Read + io::Seek + io::Write + Info {}
