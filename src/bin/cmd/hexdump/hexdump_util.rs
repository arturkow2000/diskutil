use std::cmp::min;
use std::io::{self, Read};

pub struct Options {
    pub print_offset: bool,
    pub ascii_dump: bool,
    pub verbose: bool,
    pub words_per_row: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            print_offset: true,
            ascii_dump: true,
            verbose: false,
            words_per_row: 16,
        }
    }
}

fn ascii_dump(line: &[u8]) {
    print!(" |");

    for &b in line {
        let c = b as char;
        if c.is_ascii_graphic() {
            print!("{}", c)
        } else {
            print!(".");
        }
    }

    print!("|");
}

fn hexdump_row(address: usize, buf: &[u8], opt: &Options) {
    let mut left = buf.len();
    let mut offset = 0;

    if opt.print_offset {
        print!("{:08x}  ", address);
    }

    while left > 0 {
        if offset > 0 {
            print!(" ");
        }

        if offset >= (opt.words_per_row / 2) && offset % (opt.words_per_row / 2) == 0 {
            print!(" ");
        }

        print!("{:02x}", buf[offset]);

        left -= 1;
        offset += 1;
    }

    if opt.ascii_dump {
        if buf.len() % opt.words_per_row != 0 {
            let m = opt.words_per_row - buf.len();
            let mut padding_len = m * 3;
            if m >= opt.words_per_row / 2 {
                padding_len += 1;
            }

            for _ in 0..padding_len {
                print!(" ");
            }
        }

        ascii_dump(buf);
    }

    println!()
}

pub fn hexdump_from_reader<T: Read + ?Sized>(
    reader: &mut T,
    length: usize,
    opt: &Options,
) -> io::Result<()> {
    const BLOCK_SIZE: usize = 16777216;

    // FIXME: we are currently wasting time for initializing buffer which
    // will overridden right away.
    // Currently Rust provides no safe way to read into uninitialized buffer and
    // we want to avoid using unsafe code as much as possible
    // see https://rust-lang.github.io/rfcs/2930-read-buf.html
    let mut buf = vec![0; min(length, BLOCK_SIZE)];

    let mut left = length;
    let mut address = 0;

    let mut a = false;
    let mut last_row_copy: Vec<u8> = Vec::new();

    while left > 0 {
        let n = min(BLOCK_SIZE, left);
        reader.read_exact(&mut buf[..n])?;

        if !opt.verbose {
            last_row_copy.resize(opt.words_per_row, 0);
        }

        {
            let mut left = n;
            let mut offset = 0;

            while left > 0 {
                let n = min(left, opt.words_per_row);
                let s = &buf[offset..offset + n];

                if opt.verbose {
                    hexdump_row(address, s, opt);
                    address += opt.words_per_row;
                } else if &last_row_copy[..n] == s {
                    if !a {
                        println!("*");
                        a = true;
                    }
                    address += opt.words_per_row;
                } else {
                    hexdump_row(address, s, opt);
                    address += opt.words_per_row;

                    if n < opt.words_per_row {
                        last_row_copy.resize(n, 0);
                    }

                    last_row_copy.copy_from_slice(s);
                    a = false;
                }

                left -= n;
                offset += n;
            }
        }

        left -= n;
    }

    Ok(())
}
