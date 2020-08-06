#[macro_use]
extern crate clap;
extern crate diskutil;
extern crate fern;
extern crate log;

use clap::ArgMatches;
use diskutil::disk::raw::RawDisk;
use diskutil::disk::vhd::VhdDisk;
use diskutil::disk::{Disk, MediaType};
use diskutil::part::gpt::{ErrorAction, Gpt};
use diskutil::Result;

use std::fs::{File, OpenOptions};
use std::io;

fn setup_logging(matches: &ArgMatches) {
    use fern::colors::{Color, ColoredLevelConfig};

    let colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::White)
        .debug(Color::BrightWhite)
        .trace(Color::Cyan);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            let color = colors.get_color(&record.level());
            let foreground = color.to_fg_str();
            let target = record.target();
            let level = record.level();

            let prefix = format!("[{}][{}]\x1b[{}m ", target, level, foreground);
            const SUFFIX: &'static str = "\x1b[0m";

            let s = format!("{}", message);
            let mut string_len = 0;
            let mut num_lines = 0;
            for x in s.split('\n') {
                string_len += x.len();
                num_lines += 1;
            }
            if num_lines == 0 {
                num_lines = 1;
            }

            let c = string_len + num_lines * (prefix.len() + SUFFIX.len()) + num_lines;
            let mut buf = String::with_capacity(c);
            for (i, line) in s.split('\n').enumerate() {
                buf += &prefix;
                buf += line;
                buf += SUFFIX;
                if i != num_lines - 1 {
                    buf.push('\n');
                }
            }

            debug_assert!(c >= buf.len());

            out.finish(format_args!("{}", buf))
        })
        .level(match matches.occurrences_of("verbose") {
            0 => log::LevelFilter::Off,
            1 => log::LevelFilter::Error,
            2 => log::LevelFilter::Warn,
            3 => log::LevelFilter::Info,
            4 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        })
        .chain(io::stdout())
        .apply()
        .unwrap();
}

macro_rules! arg_parse_integer {
    (usize, $m:expr, $n:expr, $def:expr) => {{
        if let Some(t) = $m.value_of($n) {
            usize::from_str_radix(&t, 10).expect("Invalid integer")
        } else {
            $def
        }
    }};
}

fn get_disk(file: File, matches: &ArgMatches) -> Result<Box<dyn Disk>> {
    let block_size = arg_parse_integer!(usize, matches, "sector_size", 512);

    let file_size = file.metadata().unwrap().len() as usize;
    let num_blocks = file_size / block_size;

    if file_size % block_size != 0 {
        panic!("File size is not multiple of {}", block_size);
    }

    // TODO: support format guessing
    let x: Box<dyn Disk> = match matches.value_of("format").unwrap().to_lowercase().as_str() {
        "raw" => Box::new(RawDisk::open(file, 512, num_blocks, MediaType::HDD)),
        "vhd" => Box::new(VhdDisk::open(file)?),
        x => panic!("Unknown format {}", x),
    };

    Ok(x)
}

fn get_subcommand_handler(command: &str) -> &'static dyn Fn(&mut Gpt, Option<&ArgMatches>) {
    match command {
        "print" => &handle_print_subcommand,
        command => panic!("Unknown subcommand: {}", command),
    }
}

fn main() -> Result<()> {
    let matches = clap_app!(partdump =>
        (@arg file: +required)
        (@arg format: -f --format +required +takes_value "Select disk format, currently supported formats: RAW, VHD")
        (@arg sector_size: --sector-size +takes_value "Select sector size for RAW disk (default 512 bytes)")
        (@arg verbose: -v --verbose ...)
        (@subcommand create =>
        )
        (@subcommand print =>
            (@arg full: -f --full)
        )
    )
    .get_matches();
    setup_logging(&matches);

    let mut disk = get_disk(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(matches.value_of("file").unwrap())?,
        &matches,
    )?;

    let (subcommand, subcommand_matches) = matches.subcommand();
    if subcommand == "create" {
        handle_create_subcommand(disk.as_mut(), subcommand_matches);
        return Ok(());
    }

    let handler = if subcommand.is_empty() {
        &handle_print_subcommand
    } else {
        get_subcommand_handler(subcommand)
    };

    let mut gpt = Gpt::load(disk.as_mut(), ErrorAction::Abort)?;

    handler(&mut gpt, subcommand_matches);

    Ok(())
}

fn handle_create_subcommand(disk: &mut dyn Disk, _matches: Option<&ArgMatches>) {
    let mut gpt = Gpt::create(disk).unwrap();
    gpt.update().unwrap();
}

fn handle_print_subcommand(_gpt: &mut Gpt, _matches: Option<&ArgMatches>) {
    println!("Subcommand handler: PRINT");
}
