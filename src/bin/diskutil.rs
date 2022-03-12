use std::path::PathBuf;

use clap::{Parser, Subcommand};
use diskutil::disk::DiskFormat;

mod cmd;
mod utils;

#[macro_use]
extern crate anyhow;

#[derive(Subcommand)]
enum Command {
    Create(cmd::create::Command),
    Gpt(cmd::gpt::Command),
    Hexdump(cmd::hexdump::Command),
    Read(cmd::read::Command),
    Write(cmd::write::Command),
    Fat(cmd::fat::Command),
}

#[derive(Parser)]
struct Options {
    #[clap(subcommand)]
    pub command: Command,

    #[clap(short, parse(from_occurrences), help = "Increase verbosity level")]
    pub verbose: u32,

    #[clap(flatten)]
    pub disk: CommonDiskOptions,
}

#[derive(Parser)]
pub struct CommonDiskOptions {
    #[cfg(feature = "device")]
    #[clap(short, long, help = "Disk type [raw, vhd, device]")]
    format: DiskFormat,
    #[cfg(not(feature = "device"))]
    #[clap(short, long, help = "Disk type [raw, vhd]")]
    format: DiskFormat,
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let o = Options::parse();

    utils::setup_logging(o.verbose);

    match o.command {
        Command::Create(c) => cmd::create::run(&o.disk, c),
        Command::Gpt(c) => cmd::gpt::run(&o.disk, c),
        Command::Hexdump(c) => cmd::hexdump::run(&o.disk, c),
        Command::Read(c) => cmd::read::run(&o.disk, c),
        Command::Write(c) => cmd::write::run(&o.disk, c),
        Command::Fat(c) => cmd::fat::run(&o.disk, c),
    }
}
