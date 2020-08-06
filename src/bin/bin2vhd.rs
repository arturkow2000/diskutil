extern crate clap;
extern crate diskutil;

use clap::clap_app;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use diskutil::disk::vhd::VhdDisk;
use diskutil::disk::Info;
use diskutil::Result;

fn main() -> Result<()> {
    let matches = clap_app!(bin2vhd =>
        (@arg INPUT: +required)
        (@arg OUTPUT: +required)
    )
    .get_matches();

    let input_path = matches.value_of("INPUT").unwrap();
    let output_path = matches.value_of("OUTPUT").unwrap();

    let mut input = OpenOptions::new()
        .read(true)
        .write(false)
        .open(input_path)?;
    let mut output = File::create(output_path)?;

    let mut vhd =
        VhdDisk::create_dynamic(&mut output, 1024 * 1024 * 1024 * 64 / 512 /* 64 GiB */)?;

    let mut buf: Vec<u8> = Vec::new();
    buf.reserve(1024 * 1024 * 16);
    unsafe { buf.set_len(buf.capacity()) };

    let block_size = vhd.block_size();
    assert_eq!(buf.len() % block_size, 0);

    loop {
        let n = input.read(buf.as_mut_slice())?;
        if n == 0 {
            break;
        }
        vhd.write_all(&buf[..n])?;
    }

    Ok(())
}
