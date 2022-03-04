use std::cmp::min;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Instant;

use crate::{
    utils::{display_progress, get_partition_region, open_disk, AccessMode, PartitionId},
    CommonDiskOptions,
};
use anyhow::Context;
use clap::{ArgGroup, Parser};
use diskutil::disk::{Disk, DiskSlice};
use diskutil::part::load_partition_table;

#[derive(Parser)]
#[clap(group = ArgGroup::new("grp_offset").required(true))]
#[clap(group = ArgGroup::new("grp_length").required(false))]
#[clap(about = "Read raw data from disk")]
pub struct Command {
    #[clap(flatten)]
    disk: CommonDiskOptions,

    #[clap(
        short,
        group = "grp_offset",
        help = "Offset in bytes where to read from, relative to selected partition"
    )]
    offset: Option<u64>,

    #[clap(
        short,
        group = "grp_offset",
        help = "Offset in sectors where to read from, relative to selected partition"
    )]
    sector: Option<u64>,

    #[clap(
        short = 'l',
        group = "grp_length",
        value_name = "NUMBER OF BYTES",
        help = "Number of bytes to read"
    )]
    length_in_bytes: Option<u64>,

    #[clap(
        short = 'n',
        group = "grp_length",
        value_name = "NUMBER OF SECTORS",
        help = "Number of sectors to read"
    )]
    length_in_sectors: Option<u64>,

    #[clap(short = 'p', help = "Partition to read from")]
    partition: Option<PartitionId>,

    #[clap(long = "out", help = "Output file")]
    output: Option<PathBuf>,

    #[clap(long)]
    progress: bool,
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

    fn get_output_stream(&self) -> anyhow::Result<Box<dyn Write>> {
        if let Some(path) = self.output.as_deref() {
            Ok(Box::new(
                OpenOptions::new()
                    .read(false)
                    .write(true)
                    .create(true)
                    .open(path)
                    .context("failed to open output file")?,
            ))
        } else {
            Ok(Box::new(std::io::stdout()))
        }
    }
}

pub fn run(command: Command) -> anyhow::Result<()> {
    let mut disk = open_disk(
        command.disk.file.as_path(),
        command.disk.format,
        AccessMode::ReadOnly,
    )?;

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

    let mut out = command.get_output_stream()?;

    // FIXME: we are currently wasting time for initializing buffer which
    // will overridden right away.
    // Currently Rust provides no safe way to read into uninitialized buffer and
    // we want to avoid using unsafe code as much as possible
    // see https://rust-lang.github.io/rfcs/2930-read-buf.html
    let mut buf = vec![0; min(length.try_into().unwrap_or(usize::MAX), 16777216)];

    part.seek(SeekFrom::Start(offset)).context("seek failed")?;

    let mut left = length;
    while left > 0 {
        let start_time = Instant::now();
        let n = min(left.try_into().unwrap_or(usize::MAX), buf.len());
        part.read_exact(&mut buf[..n]).context("read failed")?;
        out.write_all(&buf[..n]).context("write failed")?;
        left -= n as u64;

        if command.progress {
            let end_time = Instant::now();
            let duration = end_time.duration_since(start_time);
            let bytes_per_second = n as f64 / duration.as_secs_f64();
            display_progress(left, length, bytes_per_second);
        }
    }

    Ok(())
}
