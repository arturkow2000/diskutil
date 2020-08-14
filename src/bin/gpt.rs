mod utils;

use clap::Clap;
use diskutil::disk::{open_disk, Disk, DiskFormat, FileBackend};
use diskutil::part::{
    gpt::{uuid128_partition_type_guid_to_name, ErrorAction, Gpt, GptPartition, GptPartitionType},
    mbr::Mbr,
};
use diskutil::region::Region;
use diskutil::Result;
use std::fs::OpenOptions;
use std::mem::transmute;
use std::path::PathBuf;
use std::result;

fn parse_sector_size(x: &str) -> result::Result<usize, String> {
    let x = usize::from_str_radix(x, 10).map_err(|e| e.to_string())?;
    if !x.is_power_of_two() {
        return Err("sector size not power of 2".to_owned());
    }

    Ok(x)
}

#[derive(Clap)]
struct Options {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u32,

    #[clap(name = "file", parse(from_os_str))]
    pub file: PathBuf,

    #[clap(long, name = "sector_size", parse(try_from_str = parse_sector_size), default_value = "512", long_about = "Set sector size for RAW disks, for other disk formats this is ignored.")]
    pub sector_size: usize,

    #[clap(long, parse(try_from_str))]
    pub disk_format: DiskFormat,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(long_about = "Create new partition table")]
    Create,
    Print,
    #[clap(long_about = "Add new partition")]
    Add(AddOptions),
}

#[derive(Clap)]
struct AddOptions {
    pub start: u64,
    #[clap(parse(try_from_str = utils::parse_size))]
    pub size: u64,
}

fn main() -> Result<()> {
    better_panic::install();
    let options = Options::parse();
    utils::setup_logging(options.verbose);

    // TODO: pass sector_size
    let mut disk = open_disk(
        options.disk_format,
        FileBackend::new(
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(options.file)?,
        )?,
        Default::default(),
    )?;

    if let SubCommand::Create = options.subcommand {
        Mbr::create_protective(disk.as_mut()).update()?;
        Gpt::create(disk.as_mut())?.update(disk.as_mut())?;
        return Ok(());
    }

    let mut gpt = Gpt::load(disk.as_mut(), ErrorAction::Abort)?;

    match options.subcommand {
        SubCommand::Create => unreachable!(),
        SubCommand::Print => print_patition_table(disk.as_ref(), &gpt)?,
        SubCommand::Add(options) => add_partition(disk.as_mut(), &mut gpt, &options)?,
    };

    Ok(())
}

fn print_patition_table(disk: &dyn Disk, gpt: &Gpt) -> Result<()> {
    println!("{:<8} {:<8} {:<8} {:<45}", "Start", "End", "Size", "Type");
    for p in &gpt.partitions {
        if let Some(p) = p {
            if let Some(size) = p
                .end_lba
                .checked_sub(p.start_lba)
                .map(|x| (x + 1).saturating_mul(disk.block_size().into()))
            {
                let t = uuid128_partition_type_guid_to_name(unsafe {
                    transmute(p.type_guid.as_u128())
                });

                if let Some(t) = t {
                    println!(
                        "{:<8} {:<8} {:<8} {:<45}",
                        p.start_lba,
                        p.end_lba,
                        utils::size_to_string(size),
                        t
                    );
                } else {
                    println!(
                        "{:<8} {:<8} {:<8} {:<45}",
                        p.start_lba,
                        p.end_lba,
                        utils::size_to_string(size),
                        p.type_guid.to_string()
                    );
                }
            } else {
                println!("{:<8} {:<8} ERROR: end < start", p.start_lba, p.end_lba);
            }
        }
    }
    Ok(())
}

fn add_partition(disk: &mut dyn Disk, gpt: &mut Gpt, options: &AddOptions) -> Result<()> {
    let sector_size = disk.block_size() as u64;
    if options.size % sector_size != 0 {
        panic!("Partition size is not multiple of {}", sector_size);
    }

    let usable_region = Region::new(gpt.first_usable_lba, gpt.last_usable_lba);
    let new_part_region = Region::new_with_size(options.start, options.size / sector_size);

    if !new_part_region.belongs(&usable_region) {
        panic!(
            "Region {} does not belong to {}",
            new_part_region, usable_region
        );
    }

    for (i, p) in gpt.partitions.iter().enumerate() {
        if let Some(p) = p {
            let r = Region::new(p.start_lba, p.end_lba);
            if new_part_region.overlaps(&r) {
                panic!(
                    "New partition would overlap with #{}: {} overlaps with {}",
                    i, new_part_region, r
                );
            }
        }
    }

    let free_entry_index = gpt
        .partitions
        .iter()
        .enumerate()
        .find(|(_, x)| x.is_none())
        .map(|(i, _)| i)
        .expect("No free partition slot found.");

    gpt.partitions[free_entry_index] = Some(GptPartition::new(
        GptPartitionType::MicrosoftBasicData,
        "blablabla",
        new_part_region.start(),
        new_part_region.end(),
    ));

    gpt.update(disk)?;

    Ok(())
}
