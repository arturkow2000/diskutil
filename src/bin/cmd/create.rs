use std::cmp::min;
use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::Write;

use crate::{utils::parse_size, CommonDiskOptions};
use anyhow::Context;
use clap::Parser;
use diskutil::disk::vhd::{DiskType as VhdDiskType, VhdDisk};
use diskutil::disk::DiskFormat;
use diskutil::disk::FileBackend;

#[derive(Parser)]
#[clap(about = "Create disk images")]
pub struct Command {
    #[clap(
        short = 's',
        long = "static",
        help = "Create statically sized disk, by default dynamically sized disk is created if supported by disk format."
    )]
    pub statically_sized: bool,

    #[clap(parse(try_from_str = parse_size))]
    pub size: u64,
}

pub fn create_vhd(file: File, size: u64, disk_type: VhdDiskType) -> anyhow::Result<()> {
    let b = FileBackend::new(file).context("failed to initialize backend")?;
    if disk_type == VhdDiskType::Dynamic {
        VhdDisk::create_dynamic(b, size.try_into().unwrap())
            .map(|_| ())
            .context("failed to create VHD disk")
    } else {
        unimplemented!()
    }
}

pub fn create_raw(mut file: File, size: u64) -> anyhow::Result<()> {
    let zero = vec![0; 65536];

    // FIXME: should allocate space using OS specific calls
    // Zeroing may be unnecessary and it contributes to disk fragmentation and
    // wear-out.
    let mut total_left = size;
    while total_left > 0 {
        let n = min(zero.len(), total_left.try_into().unwrap_or(usize::MAX));
        file.write_all(&zero[..n])?;
        total_left -= n as u64;
    }

    file.flush()?;

    Ok(())
}

pub fn run(disk: &CommonDiskOptions, command: Command) -> anyhow::Result<()> {
    match disk.format {
        DiskFormat::RAW => create_raw(
            OpenOptions::new()
                .read(false)
                .write(true)
                .create_new(true)
                .open(&disk.file)
                .context("failed to create file")?,
            command.size,
        ),
        DiskFormat::VHD => create_vhd(
            OpenOptions::new()
                .read(false)
                .write(true)
                .create_new(true)
                .open(&disk.file)
                .context("failed to create file")?,
            command.size,
            if command.statically_sized {
                VhdDiskType::Fixed
            } else {
                VhdDiskType::Dynamic
            },
        ),
        t => bail!("unsupported disk type {}", t),
    }
}
