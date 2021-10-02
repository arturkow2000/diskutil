use std::path::PathBuf;

use clap::{Clap, Subcommand};
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
}

#[derive(Clap)]
struct Options {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Clap)]
struct CommonDiskOptions {
    #[clap(short, long)]
    format: DiskFormat,
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let o = Options::parse();

    match o.command {
        Command::Create(c) => cmd::create::run(c),
        Command::Gpt(c) => cmd::gpt::run(c),
        Command::Hexdump(c) => cmd::hexdump::run(c),
        Command::Read(c) => cmd::read::run(c),
        Command::Write(c) => cmd::write::run(c),
    }
}
