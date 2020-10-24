extern crate better_panic;
extern crate clap;
extern crate diskutil;

use clap::Clap;
use diskutil::Result;
use std::cmp::min;
use std::convert::TryInto;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

mod utils;

#[derive(Clap)]
struct Options {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u32,

    #[clap(name = "file", parse(from_os_str))]
    pub file: PathBuf,

    #[clap(name = "size", parse(try_from_str = utils::parse_size))]
    pub size: u64,
}

fn main() -> Result<()> {
    better_panic::install();
    let options = Options::parse();
    utils::setup_logging(options.verbose);

    let mut file = File::create(options.file)?;
    let zero = vec![0; 65536];

    let mut total_left = options.size;
    while total_left > 0 {
        let n = min(zero.len(), total_left.try_into().unwrap_or(usize::MAX));
        file.write_all(&zero[..n])?;
        total_left -= n as u64;
    }

    file.flush()?;

    Ok(())
}
