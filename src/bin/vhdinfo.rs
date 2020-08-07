extern crate better_panic;
extern crate diskutil;

use std::env::args;
use std::fs::File;

use diskutil::disk::vhd::VhdDisk;
use diskutil::disk::Info;
use diskutil::Result;

fn main() -> Result<()> {
    better_panic::install();
    let mut args = args();
    args.next().unwrap();

    let mut file = File::open(args.next().expect("Usage: vhdinfo file"))?;
    let disk = VhdDisk::open(&mut file)?;

    let max_disk_size = disk.max_disk_size();

    println!("Max disk size    : {}", max_disk_size);

    Ok(())
}
