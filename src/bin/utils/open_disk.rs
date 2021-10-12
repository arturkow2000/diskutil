use std::fs::OpenOptions;
use std::path::Path;

use anyhow::Context;
use diskutil::disk::{self, ArgumentMap, Backend, Disk, DiskFormat, FileBackend};

#[cfg(feature = "device")]
use diskutil::disk::DeviceBackend;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AccessMode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl AccessMode {
    pub fn can_read(self) -> bool {
        self == Self::ReadOnly || self == Self::ReadWrite
    }

    pub fn can_write(self) -> bool {
        self == Self::WriteOnly || self == Self::ReadWrite
    }
}

pub fn open_disk(
    path: &Path,
    format: DiskFormat,
    access: AccessMode,
) -> anyhow::Result<Box<dyn Disk>> {
    let backend: Box<dyn Backend> = if format == DiskFormat::Device {
        #[cfg(feature = "device")]
        {
            DeviceBackend::new(
                path,
                if access == AccessMode::ReadOnly {
                    false
                } else {
                    true
                },
            )
            .context("failed to open device")?
        }
        #[cfg(not(feature = "device"))]
        {
            anyhow::bail!("physical device support was not enabled when compiling")
        }
    } else {
        FileBackend::new(
            OpenOptions::new()
                .read(access.can_read())
                .write(access.can_write())
                .open(path)
                .context("failed to open file")?,
        )
        .context("failed to create disk backend (is this a regular file?)")?
    };

    Ok(disk::open_disk(format, backend, ArgumentMap::default())?)
}
