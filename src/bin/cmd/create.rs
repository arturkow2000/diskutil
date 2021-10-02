use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use crate::utils::parse_size;
use anyhow::Context;
use clap::Clap;
use diskutil::disk::vhd::{DiskType as VhdDiskType, VhdDisk};
use diskutil::disk::DiskFormat;
use diskutil::disk::FileBackend;

#[derive(Clap)]
#[clap(about = "Create disk images")]
pub struct Command {
    #[clap(short, long)]
    pub format: DiskFormat,

    #[clap(
        short = 's',
        long = "static",
        about = "Create statically sized disk, by default dynamically sized disk is created if supported by disk format."
    )]
    pub statically_sized: bool,

    pub file: PathBuf,

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

pub fn run(command: Command) -> anyhow::Result<()> {
    match command.format {
        DiskFormat::RAW => {
            unimplemented!()
        }
        DiskFormat::VHD => create_vhd(
            OpenOptions::new()
                .read(false)
                .write(true)
                .create_new(true)
                .open(command.file)
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
