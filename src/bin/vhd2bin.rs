extern crate better_panic;
extern crate clap;
extern crate diskutil;

use std::env::args;
use std::fs::File;
use std::io::{Read, Write};

use diskutil::disk::vhd::VhdDisk;
use diskutil::disk::{FileBackend, Info};
use diskutil::Result;

fn main() -> Result<()> {
    better_panic::install();
    let mut args = args();
    args.next().unwrap();

    let input = FileBackend::new(File::open(
        args.next().expect("Usage: vhd2bin input output"),
    )?)?;
    let mut output = File::create(args.next().expect("Usage: vhd2bin input output"))?;

    let mut disk = VhdDisk::open(input)?;

    let mut buf: Vec<u8> = Vec::new();
    buf.reserve(1024 * 1024 * 16);
    unsafe { buf.set_len(buf.capacity()) };

    let block_size = disk.block_size();
    assert_eq!(buf.len() % block_size as usize, 0);

    loop {
        let n = disk.read(buf.as_mut_slice())?;
        if n == 0 {
            break;
        }
        output.write_all(&buf[..n])?;
    }

    Ok(())
}
