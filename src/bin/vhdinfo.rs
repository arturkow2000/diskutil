extern crate better_panic;
extern crate diskutil;

use std::env::args;
use std::fs::File;

use diskutil::disk::vhd::VhdDisk;
use diskutil::disk::FileBackend;
use diskutil::Result;

fn main() -> Result<()> {
    better_panic::install();
    let mut args = args();
    args.next().unwrap();

    let file = FileBackend::new(File::open(args.next().expect("Usage: vhdinfo file"))?)?;
    let _disk = VhdDisk::open(file)?;

    unimplemented!()
}
