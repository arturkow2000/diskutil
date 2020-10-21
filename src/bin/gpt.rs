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
use std::str::FromStr;
use uuid::Uuid;

fn parse_sector_size(x: &str) -> result::Result<usize, String> {
    let x = usize::from_str_radix(x, 10).map_err(|e| e.to_string())?;
    if !x.is_power_of_two() {
        return Err("sector size not power of 2".to_owned());
    }

    Ok(x)
}

fn parse_partition_type(s: &str) -> result::Result<Uuid, String> {
    if s.chars().next().map_or(false, |x| x == '{')
        && s.chars().rev().next().map_or(false, |x| x == '}')
    {
        Ok(Uuid::from_str(&s[1..s.len() - 1]).map_err(|e| e.to_string())?)
    } else {
        match s.to_lowercase().as_str() {
            "msbasic" => Ok(GptPartitionType::MicrosoftBasicData.to_guid()),
            "msreserved" => Ok(GptPartitionType::MicrosoftReserved.to_guid()),
            "efi" | "esp" => Ok(GptPartitionType::EFISystemPartition.to_guid()),
            _ => Err("Unknown partition type".to_owned()),
        }
    }
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
    #[clap(long_about = "Delete partition")]
    Delete(DeleteOptions),
    #[clap(long_about = "Modify partition")]
    Modify(ModifyOptions),
}

#[derive(Clap)]
struct AddOptions {
    pub start: u64,
    #[clap(parse(try_from_str = utils::parse_size), long_about = "Partition start in sectors")]
    pub size: u64,
    #[clap(short, long)]
    pub name: Option<String>,
    #[clap(short = 'u', long = "guid", long_about = "Unique GUID")]
    pub unique_guid: Option<Uuid>,
    #[clap(short = 't', long = "type", parse(try_from_str = parse_partition_type), long_about = "Type GUID or one of: msbasic, msreserved, esp")]
    pub type_guid: Option<Uuid>,
}

#[derive(Clap)]
struct DeleteOptions {
    #[clap(parse(try_from_str))]
    pub id: utils::PartitionId,
}

#[derive(Clap)]
struct ModifyOptions {
    #[clap(parse(try_from_str))]
    pub partition: utils::PartitionId,
    #[clap(short, long)]
    pub name: Option<String>,
    #[clap(short = 'u', long = "guid", long_about = "Unique GUID")]
    pub unique_guid: Option<Uuid>,
    #[clap(short = 't', long = "type", parse(try_from_str = parse_partition_type), long_about = "Type GUID or one of: msbasic, msreserved, esp")]
    pub type_guid: Option<Uuid>,
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
        SubCommand::Print => print_partition_table(disk.as_ref(), &gpt)?,
        SubCommand::Add(options) => add_partition(disk.as_mut(), &mut gpt, &options)?,
        SubCommand::Delete(options) => delete_partition(disk.as_mut(), &mut gpt, &options)?,
        SubCommand::Modify(options) => modify_partition(disk.as_mut(), &mut gpt, &options)?,
    };

    Ok(())
}

fn print_partition_table(disk: &dyn Disk, gpt: &Gpt) -> Result<()> {
    println!(
        "{:<5} {:<8} {:<8} {:<8} {:<38} {:<45} Name",
        "Index", "Start", "End", "Size", "Unique GUID", "Type"
    );
    for (i, p) in gpt
        .partitions
        .iter()
        .enumerate()
        .map(|(i, x)| (i, x.as_ref()))
    {
        if let Some(p) = p {
            if let Some(size) = p
                .end_lba
                .checked_sub(p.start_lba)
                .map(|x| (x + 1).saturating_mul(disk.sector_size().into()))
            {
                let t = uuid128_partition_type_guid_to_name(unsafe {
                    transmute(p.type_guid.as_u128())
                });

                if let Some(t) = t {
                    println!(
                        "{:<5} {:<8} {:<8} {:<8} {{{:<38X}}} {:<45} {}",
                        i,
                        p.start_lba,
                        p.end_lba,
                        utils::size_to_string(size),
                        p.unique_guid,
                        t,
                        &p.partition_name
                    );
                } else {
                    println!(
                        "{:<5} {:<8} {:<8} {:<8} {{{:<38X}}} {:<45} {}",
                        i,
                        p.start_lba,
                        p.end_lba,
                        utils::size_to_string(size),
                        p.unique_guid,
                        p.type_guid.to_string(),
                        &p.partition_name
                    );
                }
            } else {
                println!(
                    "{:<5} {:<8} {:<8} ERROR: end < start",
                    i, p.start_lba, p.end_lba
                );
            }
        }
    }
    Ok(())
}

fn add_partition(disk: &mut dyn Disk, gpt: &mut Gpt, options: &AddOptions) -> Result<()> {
    let sector_size = disk.sector_size() as u64;
    if options.size % sector_size != 0 {
        panic!("Partition size is not multiple of {}", sector_size);
    }

    // TODO: move verification into library
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

    gpt.partitions[free_entry_index] = Some(GptPartition::new_ex(
        options
            .type_guid
            .unwrap_or(GptPartitionType::MicrosoftBasicData.to_guid()),
        if let Some(n) = options.name.as_ref() {
            n.as_ref()
        } else {
            ""
        },
        new_part_region.start(),
        new_part_region.end(),
        options.unique_guid.unwrap_or_else(|| Uuid::new_v4()),
    ));

    gpt.update(disk)?;

    Ok(())
}

fn delete_partition(disk: &mut dyn Disk, gpt: &mut Gpt, options: &DeleteOptions) -> Result<()> {
    let partition_index = match options.id {
        utils::PartitionId::Index(i) => {
            gpt.get_partition(i).expect("No such partition.");
            i
        }
        utils::PartitionId::Guid(g) => gpt.find_partition_by_guid(g).expect("No such partition.").0,
    };

    gpt.partitions[partition_index as usize] = None;
    gpt.update(disk)?;

    Ok(())
}

fn modify_partition(disk: &mut dyn Disk, gpt: &mut Gpt, options: &ModifyOptions) -> Result<()> {
    let partition = match options.partition {
        utils::PartitionId::Index(i) => gpt.get_partition_mut(i).expect("No such partition."),
        utils::PartitionId::Guid(g) => {
            gpt.find_partition_by_guid_mut(g)
                .expect("No such partition.")
                .1
        }
    };

    if let Some(name) = options.name.as_ref() {
        partition.partition_name = name.clone();
    }

    if let Some(unique_guid) = options.unique_guid.as_ref() {
        partition.unique_guid = unique_guid.clone();
    }

    if let Some(type_guid) = options.type_guid.as_ref() {
        partition.type_guid = type_guid.clone();
    }

    gpt.update(disk)?;

    Ok(())
}
