use better_panic;
use clap::Clap;
use diskutil::disk::{open_disk, DiskFormat, FileBackend};
use diskutil::part::{
    gpt::{ErrorAction, Gpt},
    mbr::Mbr,
};
use diskutil::Result;
use std::fs::File;
use std::path::PathBuf;
use std::result;

mod utils;

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
    Create,
    Print,
}

fn main() -> Result<()> {
    better_panic::install();
    let options = Options::parse();
    utils::setup_logging(options.verbose);

    let mut disk = open_disk(
        options.disk_format,
        FileBackend::new(File::open(options.file)?)?,
        Default::default(),
    )?;

    match options.subcommand {
        SubCommand::Create => {
            Mbr::create_protective(disk.as_mut()).update()?;
            Gpt::create(disk.as_mut())?.update(disk.as_mut())?;
            return Ok(());
        }
        _ => (),
    };

    let _gpt = Gpt::load(disk.as_mut(), ErrorAction::Abort)?;

    Ok(())
}
