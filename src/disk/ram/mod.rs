use crate::disk::{Disk, DiskFormat, Info, MediaType};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

pub struct RamDisk {
    buffer: Cursor<Vec<u8>>,
    sector_size: u32,
    media_type: MediaType,
}

impl RamDisk {
    pub fn new_uninitialized(sector_size: u32, num_sectors: u32) -> Self {
        let size_in_bytes = sector_size as usize * num_sectors as usize;
        let mut buffer = Vec::with_capacity(size_in_bytes);
        unsafe {
            buffer.set_len(size_in_bytes);
        }

        Self {
            buffer: Cursor::new(buffer),
            sector_size,
            media_type: MediaType::HDD,
        }
    }

    pub fn new_zeroed(sector_size: u32, num_sectors: u32) -> Self {
        let size_in_bytes = sector_size as usize * num_sectors as usize;
        let buffer = vec![0u8; size_in_bytes];

        Self {
            buffer: Cursor::new(buffer),
            sector_size,
            media_type: MediaType::HDD,
        }
    }

    pub fn from_vec(vector: Vec<u8>, sector_size: u32) -> Self {
        assert_eq!(vector.len() % sector_size as usize, 0);

        Self {
            buffer: Cursor::new(vector),
            sector_size,
            media_type: MediaType::HDD,
        }
    }
}

impl Read for RamDisk {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.buffer.read(buf)
    }
}

impl Seek for RamDisk {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.buffer.seek(pos)
    }
}

impl Write for RamDisk {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
}

impl Info for RamDisk {
    fn disk_format(&self) -> DiskFormat {
        DiskFormat::RAW
    }
    fn max_disk_size(&self) -> u64 {
        self.buffer.get_ref().len() as u64
    }
    fn disk_size(&self) -> u64 {
        self.buffer.get_ref().len() as u64
    }
    fn block_size(&self) -> u32 {
        self.sector_size
    }
    fn media_type(&self) -> MediaType {
        self.media_type
    }
}

impl Disk for RamDisk {}
