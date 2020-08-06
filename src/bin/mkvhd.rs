use std::env::args;
use std::fs::File;

extern crate diskutil;

use diskutil::disk::vhd::VhdDisk;
use diskutil::Result;

fn main() -> Result<()> {
    let mut args = args();
    args.next().unwrap();

    let mut file = File::create(args.next().expect("Usage: mkvhd file"))?;
    let _disk = VhdDisk::create_dynamic(&mut file, 1024 * 1024 * 1024 * 8 / 512 /*8G disk*/)?;

    Ok(())
}
