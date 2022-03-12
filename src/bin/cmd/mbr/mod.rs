use std::{
    fs::OpenOptions,
    io::{Read, SeekFrom},
    path::PathBuf,
};

use anyhow::Context;
use clap::Parser;
use diskutil::part::mbr::Mbr;

use crate::{
    utils::{self, open_disk, AccessMode},
    CommonDiskOptions,
};

const MBR_MAX_BOOTLOADER_SIZE: usize = 446;

#[derive(Parser)]
pub enum SubCommand {
    #[clap(about = "Dump raw contents of partition table")]
    Dump,
    #[clap(about = "Install bootloader into MBR")]
    Bootcode { file: PathBuf },
}

#[derive(Parser)]
#[clap(about = "Manipulate MBR partition table")]
pub struct Command {
    #[clap(subcommand)]
    cmd: SubCommand,
}

pub fn run(disk: &CommonDiskOptions, command: Command) -> anyhow::Result<()> {
    let mut disk = open_disk(
        disk.file.as_path(),
        disk.format,
        // TODO: select proper access mode
        AccessMode::ReadWrite,
    )?;

    let mbr = Mbr::load(disk.as_mut()).context("failed to load MBR")?;
    match command.cmd {
        SubCommand::Dump => {
            println!(
                "{:<5} {:<8} {:<8} {:<8} {:<8} {:<5}",
                "Index", "Start", "End", "Size", "Type", "Flags"
            );
            for (i, p) in mbr.partitions.iter().enumerate() {
                if let Some(p) = p.as_ref() {
                    let start = p.start();
                    let end = p.end();

                    println!(
                        "{:<5} {:<8} {:<8} {:<8} 0x{:02X}     0x{:02X}",
                        i,
                        start,
                        end,
                        utils::size_to_string(p.size() as u64 * p.sector_size as u64),
                        p.partition_type,
                        p.flags
                    );
                } else {
                    println!("{:<5} UNUSED", i);
                }
            }
        }
        SubCommand::Bootcode { file } => {
            let mut file = OpenOptions::new().read(true).write(false).open(&file)?;
            // FIXME: this won't work properly on special devices like pipes,
            // network streams, etc.
            let len = file.metadata()?.len();

            // FIXME: this works for MBR on hard drives but on floppies it would
            // corrupt BPB. On floppies MBR is actually PBR (there is no
            // partitions) and it depends on the underlying file system.
            if len > MBR_MAX_BOOTLOADER_SIZE as u64 {
                anyhow::bail!("Bootloader is to big to fit into MBR");
            }

            let mut buf = [0u8; 512];

            // Dont use Mbr class. Mbr does not support writing MBR currently.
            // But, even if it would, using it would rewrite partition table due
            // to re-encoding.
            disk.seek(SeekFrom::Start(0))?;
            disk.read_exact(&mut buf[..])?;

            file.read_exact(&mut buf[..len as usize])?;
            disk.seek(SeekFrom::Start(0))?;
            disk.write_all(&mut buf[..])?;
            disk.flush()?;
        }
    }

    Ok(())
}
