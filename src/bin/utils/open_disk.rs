use std::fs::OpenOptions;
use std::path::Path;

use anyhow::Context;
use diskutil::disk::{self, ArgumentMap, Disk, DiskFormat, FileBackend};

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
    let backend = FileBackend::new(
        OpenOptions::new()
            .read(access.can_read())
            .write(access.can_write())
            .open(path)
            .context("failed to open file")?,
    )
    .context("failed to create disk backend (is this a regular file?)")?;

    Ok(disk::open_disk(format, backend, ArgumentMap::default())?)
}
