extern crate fatfs;

use chrono::{DateTime, Local};
use clap::Parser;
use diskutil::disk::DiskSlice;
use diskutil::part::load_partition_table;
use std::cmp::min;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::result;

#[cfg(feature = "device")]
use diskutil::disk::DeviceBackend;

use crate::{
    utils::{open_disk, AccessMode, PartitionId},
    CommonDiskOptions,
};

fn parse_sector_size(x: &str) -> result::Result<usize, String> {
    let x = x.parse::<usize>().map_err(|e| e.to_string())?;
    if !x.is_power_of_two() {
        return Err("sector size not power of 2".to_owned());
    }

    Ok(x)
}

fn parse_fat_type(x: &str) -> result::Result<fatfs::FatType, &'static str> {
    match x {
        "12" => Ok(fatfs::FatType::Fat12),
        "16" => Ok(fatfs::FatType::Fat16),
        "32" => Ok(fatfs::FatType::Fat32),
        _ => Err("Unknown FAT type, expected 12, 16 or 32"),
    }
}

#[derive(Parser)]
#[clap(about = "Access FAT filesystem")]
pub struct Command {
    #[clap(flatten)]
    disk: CommonDiskOptions,

    #[clap(long, name = "sector_size", parse(try_from_str = parse_sector_size), default_value = "512", long_help = "Set sector size for RAW disks, for other disk formats this is ignored.")]
    pub sector_size: usize,

    #[clap(short = 'p', long = "partition", parse(try_from_str))]
    pub partition: Option<PartitionId>,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Parser)]
pub enum SubCommand {
    #[clap(alias = "ls")]
    Dir(SubCommandDirCat),
    #[clap(alias = "type")]
    Cat(SubCommandDirCat),
    #[clap(alias = "copy_from")]
    CopyFrom(SubCommandCopy),
    #[clap(alias = "copy_to")]
    CopyTo(SubCommandCopy),
    Format(SubCommandFormat),
    #[clap(alias = "rm")]
    #[clap(alias = "del")]
    Delete(SubCommandDelete),
    #[clap(name = "mkdir")]
    MkDir(SubCommandMkDir),
}

#[derive(Parser)]
pub struct SubCommandDirCat {
    pub path: PathBuf,
}

#[derive(Parser)]
pub struct SubCommandCopy {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Parser)]
pub struct SubCommandFormat {
    #[clap(short = 'F')]
    #[clap(parse(try_from_str = parse_fat_type))]
    pub fat_type: fatfs::FatType,
}

#[derive(Parser)]
pub struct SubCommandDelete {
    pub path: PathBuf,
    #[clap(short = 'r', long = "recursive")]
    pub recursive: bool,
}

#[derive(Parser)]
pub struct SubCommandMkDir {
    pub path: PathBuf,
}

macro_rules! u8_vector_uninitialized {
    ($size:expr) => {{
        let mut v: Vec<u8> = Vec::with_capacity($size);
        unsafe { v.set_len($size) };
        v
    }};
}

pub fn run(command: Command) -> anyhow::Result<()> {
    // TODO: pass sector_size
    let mut disk = open_disk(
        command.disk.file.as_path(),
        command.disk.format,
        AccessMode::ReadWrite,
    )?;

    let mut slice = if let Some(partition) = command.partition {
        let pt = load_partition_table(disk.as_mut()).unwrap();
        let (s, e) = match partition {
            PartitionId::Index(i) => pt.get_partition_start_end(i).unwrap(),
            PartitionId::Guid(g) => {
                let x = pt.find_partition_by_guid(g).unwrap();
                (x.1.start(), x.1.end())
            }
        };
        DiskSlice::new(disk.as_mut(), s, e)
    } else {
        let size = disk.disk_size() / disk.sector_size() as u64;
        DiskSlice::new(disk.as_mut(), 0, size)
    };

    if let SubCommand::Format(p) = command.subcommand {
        return fat_format(&mut slice, &p);
    }
    let fs = fatfs::FileSystem::new(&mut slice, fatfs::FsOptions::new()).unwrap();
    let root = fs.root_dir();

    match command.subcommand {
        SubCommand::Dir(d) => {
            if d.path.parent().is_none() {
                list_directory(&root);
            } else {
                let p = convert_path(&d.path);
                list_directory(&root.open_dir(p.as_str()).unwrap());
            }
        }
        SubCommand::Cat(d) => {
            let mut file = root.open_file(&convert_path(&d.path)).unwrap();
            let file_size = file.seek(SeekFrom::End(0)).unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();
            let mut buffer = u8_vector_uninitialized!(min(
                1024 * 1024,
                file_size.try_into().unwrap_or(usize::MAX)
            ));

            let mut left = file_size;
            let stdout = ::std::io::stdout();
            while left > 0 {
                let r = file.read(buffer.as_mut_slice()).unwrap();
                if r == 0 {
                    continue;
                }
                stdout.lock().write_all(&buffer[..r]).unwrap();
                left -= r as u64;
            }
            stdout.lock().flush().unwrap();
        }
        SubCommand::CopyFrom(d) => {
            let mut input = root.open_file(&convert_path(&d.from)).unwrap();
            let file_size = input.seek(SeekFrom::End(0)).unwrap();
            input.seek(SeekFrom::Start(0)).unwrap();
            let mut buffer = u8_vector_uninitialized!(min(
                // TODO: support setting alternate buffer size
                1024 * 1024 * 16,
                file_size.try_into().unwrap_or(usize::MAX)
            ));

            let mut output = OpenOptions::new()
                .read(false)
                .write(true)
                .create(true)
                .open(d.to)
                .unwrap();

            let mut left = file_size;
            while left > 0 {
                let r = input.read(buffer.as_mut_slice()).unwrap();
                if r == 0 {
                    continue;
                }
                output.write_all(&buffer[..r]).unwrap();
                left -= r as u64;
            }
        }
        SubCommand::CopyTo(d) => {
            let mut input = OpenOptions::new()
                .read(true)
                .write(false)
                .open(d.from)
                .unwrap();
            let file_size = input.metadata().unwrap().len();
            let mut buffer = u8_vector_uninitialized!(min(
                // TODO: support setting alternate buffer size
                1024 * 1024 * 16,
                file_size.try_into().unwrap_or(usize::MAX)
            ));

            let mut output = root.create_file(&convert_path(&d.to)).unwrap();

            let mut left = file_size;
            while left > 0 {
                let r = input.read(buffer.as_mut_slice()).unwrap();
                if r == 0 {
                    continue;
                }
                output.write_all(&buffer[..r]).unwrap();
                left -= r as u64;
            }
        }
        SubCommand::Format(_) => (),
        SubCommand::Delete(p) => {
            if p.recursive {
                let mut dir = root.open_dir(&convert_path(&p.path)).unwrap();
                delete_recursive(&mut dir);
            }
            root.remove(&convert_path(&p.path)).unwrap();
        }
        SubCommand::MkDir(p) => {
            root.create_dir(&convert_path(&p.path)).unwrap();
        }
    }

    Ok(())
}

fn list_directory<IO, TP, OCC>(dir: &fatfs::Dir<'_, IO, TP, OCC>)
where
    IO: fatfs::ReadWriteSeek,
    TP: fatfs::TimeProvider,
    OCC: fatfs::OemCpConverter,
{
    for e in dir.iter().map(|r| r.unwrap()) {
        println!(
            "{} {} {}",
            DateTime::<Local>::from(e.modified())
                .format("%d.%m.%Y %H:%M")
                .to_string(),
            if e.is_dir() { "<DIR>" } else { "     " },
            e.file_name()
        );
    }
}

fn convert_path(p: &Path) -> String {
    let mut s = String::new();
    for c in p.components() {
        match c {
            Component::Prefix(prefix) => panic!(
                "Invalid path, unexpected prefix: {}",
                prefix.as_os_str().to_str().unwrap_or("<invalid>")
            ),
            Component::RootDir => s.push('/'),
            Component::CurDir | Component::ParentDir => unimplemented!(),
            Component::Normal(x) => {
                s.push_str(x.to_str().expect("Invalid path"));
                s.push('/')
            }
        }
    }
    log::trace!("Converted path: {}", s.as_str());
    s
}

fn fat_format<T>(disk: &mut T, p: &SubCommandFormat) -> anyhow::Result<()>
where
    T: Read + Seek + Write,
{
    let mut wrapper = fatfs::StdIoWrapper::from(disk);
    fatfs::format_volume(
        &mut wrapper,
        fatfs::FormatVolumeOptions::new().fat_type(p.fat_type),
    )
    .unwrap();
    Ok(())
}

fn delete_recursive<IO, TP, OCC>(dir: &mut fatfs::Dir<'_, IO, TP, OCC>)
where
    IO: fatfs::ReadWriteSeek,
    TP: fatfs::TimeProvider,
    OCC: fatfs::OemCpConverter,
{
    for e in dir.iter() {
        let e = e.unwrap();

        if !e.is_dir() {
            dir.remove(e.file_name().as_ref()).unwrap();
            if is_dir_empty(dir).unwrap() {
                break;
            }
        } else {
            if e.file_name() == "." || e.file_name() == ".." {
                continue;
            }

            log::error!("Enter: {}", e.file_name());
            todo!()
        }
    }
}

fn is_dir_empty<IO, TP, OCC>(
    dir: &fatfs::Dir<'_, IO, TP, OCC>,
) -> ::std::result::Result<bool, fatfs::Error<IO::Error>>
where
    IO: fatfs::ReadWriteSeek,
    TP: fatfs::TimeProvider,
    OCC: fatfs::OemCpConverter,
{
    for r in dir.iter() {
        let e = r?;
        let name = e.short_file_name_as_bytes();
        // ignore special entries "." and ".."
        if name != b"." && name != b".." {
            return Ok(false);
        }
    }
    Ok(true)
}
