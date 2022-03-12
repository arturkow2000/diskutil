use std::{
    convert::TryInto,
    io::{Seek, SeekFrom},
};

use crate::{
    utils::{get_partition_region, open_disk, AccessMode, PartitionId},
    CommonDiskOptions,
};
use anyhow::Context;
use clap::{ArgGroup, Parser};
use diskutil::disk::{Disk, DiskSlice};
use diskutil::part::load_partition_table;

mod hexdump_util;

#[derive(Parser)]
#[clap(group = ArgGroup::new("grp_offset").required(true))]
#[clap(group = ArgGroup::new("grp_length").required(false))]
#[clap(about = "HEX + ASCII dump, similar to hexdump tool from Unix.")]
pub struct Command {
    #[clap(
        short,
        group = "grp_offset",
        help = "Offset in bytes from where to start hexdump, relative to selected partition"
    )]
    offset: Option<u64>,

    #[clap(
        short,
        group = "grp_offset",
        help = "Offset in sectors from where to start hexdump, relative to selected partition"
    )]
    sector: Option<u64>,

    #[clap(
        short = 'l',
        group = "grp_length",
        value_name = "NUMBER OF BYTES",
        help = "Number of bytes to dump"
    )]
    length_in_bytes: Option<u64>,

    #[clap(
        short = 'n',
        group = "grp_length",
        value_name = "NUMBER OF SECTORS",
        help = "Number of sectors to dump"
    )]
    length_in_sectors: Option<u64>,

    #[clap(short = 'p', help = "Partition to dump from")]
    partition: Option<PartitionId>,
}

impl Command {
    fn get_offset(&self, disk: &dyn Disk) -> u64 {
        self.offset
            .or_else(|| self.sector.map(|x| x * disk.sector_size() as u64))
            .unwrap()
    }

    fn get_length(&self, disk: &dyn Disk) -> u64 {
        self.length_in_bytes
            .or_else(|| {
                self.length_in_sectors
                    .map(|x| x * disk.sector_size() as u64)
            })
            .unwrap_or_else(|| disk.disk_size())
    }
}

pub fn run(disk: &CommonDiskOptions, command: Command) -> anyhow::Result<()> {
    let mut disk = open_disk(disk.file.as_path(), disk.format, AccessMode::ReadOnly)?;

    let mut part = if let Some(ref part) = command.partition {
        let pt = load_partition_table(disk.as_mut()).context("failed to load partition table")?;
        let region = get_partition_region(pt.as_ref(), part)?;
        DiskSlice::new(disk.as_mut(), region.start(), region.size())
    } else {
        let num_sectors = disk.disk_size() / disk.sector_size() as u64;
        DiskSlice::new(disk.as_mut(), 0, num_sectors)
    };

    let offset = command.get_offset(&part);
    let length = command.get_length(&part);

    part.seek(SeekFrom::Start(offset)).context("seek failed")?;

    // TODO: support other modes (currently we support only canonical mode, same as in Unix hexdump)
    hexdump_util::hexdump_from_reader(
        &mut part,
        length.try_into().unwrap(),
        &hexdump_util::Options::default(),
    )
    .context("hexdump failed")?;

    Ok(())
}
