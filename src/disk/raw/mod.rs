use crate::disk::{Disk, DiskType, Info, MediaType};
use std::io::{self, Read, Seek, SeekFrom, Write};

pub struct RawDisk<B>
where
    B: Read + Seek + Write,
{
    backend: B,
    block_size: usize,
    disk_size: usize,
    media_type: MediaType,
}

impl<B> RawDisk<B>
where
    B: Read + Seek + Write,
{
    pub fn open(backend: B, block_size: usize, num_blocks: usize, media_type: MediaType) -> Self {
        Self {
            backend,
            block_size,
            disk_size: block_size * num_blocks,
            media_type,
        }
    }
}

impl<B> Read for RawDisk<B>
where
    B: Read + Seek + Write,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.backend.read(buf)
    }
}

impl<B> Seek for RawDisk<B>
where
    B: Read + Seek + Write,
{
    fn seek(&mut self, seek: SeekFrom) -> io::Result<u64> {
        self.backend.seek(seek)
    }
}

impl<B> Write for RawDisk<B>
where
    B: Read + Seek + Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.backend.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.backend.flush()
    }
}

impl<B> Info for RawDisk<B>
where
    B: Read + Seek + Write,
{
    fn disk_type(&self) -> DiskType {
        DiskType::RAW
    }
    fn max_disk_size(&self) -> usize {
        self.disk_size
    }
    fn disk_size(&self) -> usize {
        self.disk_size
    }
    fn block_size(&self) -> usize {
        self.block_size
    }
    fn media_type(&self) -> MediaType {
        self.media_type
    }
}

impl<B> Disk for RawDisk<B> where B: Read + Seek + Write {}
