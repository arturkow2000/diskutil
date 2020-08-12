#[macro_use]
extern crate log;
extern crate fatfs;

mod utils;

use chrono::{DateTime, Local};
use clap::Clap;
use diskutil::disk::{open_disk, DiskFormat, FileBackend, Region};
use diskutil::Result;
use std::cmp::min;
use std::convert::TryInto;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Component, PathBuf};
use std::result;

fn parse_sector_size(x: &str) -> result::Result<usize, String> {
    let x = usize::from_str_radix(x, 10).map_err(|e| e.to_string())?;
    if !x.is_power_of_two() {
        return Err("sector size not power of 2".to_owned());
    }

    Ok(x)
}

#[derive(Clap)]
struct Options {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u32,

    #[clap(name = "file", parse(from_os_str))]
    pub file: PathBuf,

    #[clap(long, name = "sector_size", parse(try_from_str = parse_sector_size), default_value = "512", long_about = "Set sector size for RAW disks, for other disk formats this is ignored.")]
    pub sector_size: usize,

    #[clap(long, parse(try_from_str))]
    pub disk_format: DiskFormat,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(alias = "ls")]
    Dir(SubCommandDirCat),
    #[clap(alias = "type")]
    Cat(SubCommandDirCat),
    #[clap(alias = "copy_from")]
    CopyFrom(SubCommandCopy),
    #[clap(alias = "copy_to")]
    CopyTo(SubCommandCopy),
}

#[derive(Clap)]
struct SubCommandDirCat {
    pub path: PathBuf,
}

#[derive(Clap)]
struct SubCommandCopy {
    pub from: PathBuf,
    pub to: PathBuf,
}

macro_rules! u8_vector_uninitialized {
    ($size:expr) => {{
        let mut v: Vec<u8> = Vec::with_capacity($size);
        unsafe { v.set_len($size) };
        v
    }};
}

fn main() -> Result<()> {
    better_panic::install();
    let options = Options::parse();
    utils::setup_logging(options.verbose);

    // TODO: pass sector_size
    let mut disk = open_disk(
        options.disk_format,
        FileBackend::new(
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(options.file)?,
        )?,
        Default::default(),
    )?;

    let mut fat_region = Region::new(disk.as_mut(), 0x8000, 0x1ffefff);
    let fs = fatfs::FileSystem::new(&mut fat_region, fatfs::FsOptions::new()).unwrap();
    let root = fs.root_dir();

    match options.subcommand {
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
            while left > 0 {}
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

// fatfs library accept path as string in Unix format
// this will be removed when either I modify fatfs library
// to use rust path abstraction or replace fatfs with custom
// implementation
fn convert_path(p: &PathBuf) -> String {
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
    trace!("Converted path: {}", s.as_str());
    s
}
