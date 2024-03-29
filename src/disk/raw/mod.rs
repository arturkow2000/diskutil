use std::io::{self, Read, Seek, SeekFrom, Write};

use crate::disk::{ArgumentMap, Backend, Disk, DiskFormat, MediaType};

pub struct RawDisk {
    backend: Box<dyn Backend>,
    disk_size: u64,
    sector_size: u32,
    media_type: MediaType,
}

impl RawDisk {
    pub fn open_with_argmap(backend: Box<dyn Backend>, args: &ArgumentMap) -> Self {
        Self::open(
            backend,
            args.get_u32("sector_size").unwrap_or(512),
            MediaType::HDD,
        )
    }

    pub fn open(backend: Box<dyn Backend>, sector_size: u32, media_type: MediaType) -> Self {
        let disk_size = backend.data_length();
        assert_eq!(disk_size % sector_size as u64, 0);

        Self {
            backend,
            sector_size,
            disk_size,
            media_type,
        }
    }
}

impl Read for RawDisk {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.backend.read(buf)
    }
}

impl Seek for RawDisk {
    fn seek(&mut self, seek: SeekFrom) -> io::Result<u64> {
        self.backend.seek(seek)
    }
}

impl Write for RawDisk {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.backend.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.backend.flush()
    }
}

impl Disk for RawDisk {
    fn disk_size(&self) -> u64 {
        self.disk_size
    }

    fn sector_size(&self) -> u32 {
        self.sector_size
    }

    fn media_type(&self) -> MediaType {
        self.media_type
    }

    fn disk_format(&self) -> DiskFormat {
        DiskFormat::RAW
    }
}
