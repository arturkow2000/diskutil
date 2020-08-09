extern crate better_panic;
extern crate clap;
extern crate diskutil;

use clap::Clap;

use diskutil::disk::vhd::VhdDisk;
use diskutil::disk::FileBackend;
use diskutil::Result;
use std::fs::File;
use std::path::PathBuf;
use std::result;

mod utils;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum VhdType {
    Dynamic,
    Fixed,
}
impl VhdType {
    pub fn try_parse(x: &str) -> result::Result<Self, &'static str> {
        match x {
            "dynamic" => Ok(VhdType::Dynamic),
            "fixed" => Ok(VhdType::Fixed),
            _ => Err("Unknown VHD format"),
        }
    }
}

#[derive(Clap)]
struct Options {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u32,

    #[clap(name = "file", parse(from_os_str))]
    pub file: PathBuf,

    #[clap(name = "size", parse(try_from_str = utils::parse_size))]
    pub size: usize,

    #[clap(short = 't', long, parse(try_from_str = VhdType::try_parse), default_value = "dynamic")]
    pub vhd_type: VhdType,
}

fn main() -> Result<()> {
    better_panic::install();
    let options = Options::parse();
    utils::setup_logging(options.verbose);

    let file = FileBackend::new(File::create(options.file)?)?;

    match options.vhd_type {
        VhdType::Dynamic => VhdDisk::create_dynamic(file, options.size)?,
        VhdType::Fixed => todo!("Fixed VHD disks are not implemented yet"),
    };

    Ok(())
}
