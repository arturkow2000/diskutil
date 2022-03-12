use std::str::FromStr;

use crate::{
    utils::{open_disk, parse_size, PartitionId},
    CommonDiskOptions,
};
use anyhow::Context;
use clap::{ArgEnum, Parser};
use diskutil::part::gpt::{ErrorAction, Gpt, GptPartitionType};
use uuid::Uuid;

mod access;
mod add;
mod create;
mod delete;
mod dump;
mod modify;

fn parse_partition_type(s: &str) -> ::std::result::Result<Uuid, String> {
    if s.chars().next().map_or(false, |x| x == '{')
        && s.chars().rev().next().map_or(false, |x| x == '}')
    {
        Ok(Uuid::from_str(&s[1..s.len() - 1]).map_err(|e| e.to_string())?)
    } else {
        // TODO: add aliases for more partition types
        // and add ability to print list of these aliases
        match s.to_lowercase().as_str() {
            "msbasic" => Ok(GptPartitionType::MicrosoftBasicData.to_guid()),
            "msreserved" => Ok(GptPartitionType::MicrosoftReserved.to_guid()),
            "efi" | "esp" => Ok(GptPartitionType::EFISystemPartition.to_guid()),
            _ => Err("Unknown partition type".to_owned()),
        }
    }
}

#[derive(Copy, Clone, ArgEnum)]
pub enum MbrCreateMode {
    Protective,
}

#[derive(Parser)]
pub struct CreateOptions {
    #[clap(
        arg_enum,
        long = "mbr",
        default_value = "protective",
        help = "Select MBR type to create"
    )]
    mbr_mode: MbrCreateMode,
}

#[derive(Parser)]
pub struct AddOptions {
    #[clap(parse(try_from_str = parse_size))]
    size: u64,

    #[clap(short, long, help = "Partition first sector")]
    start: Option<u64>,

    #[clap(short, long)]
    name: Option<String>,

    #[clap(short = 'u', long = "guid", help = "Unique GUID")]
    unique_guid: Option<Uuid>,

    #[clap(short = 't', long = "type", parse(try_from_str = parse_partition_type), long_help = "Type GUID or type alias eg. msbasic, msreserved, esp")]
    type_guid: Option<Uuid>,
}

#[derive(Parser)]
pub struct DeleteOptions {
    #[clap(parse(try_from_str))]
    id: PartitionId,
}

#[derive(Parser)]
pub struct ModifyOptions {
    #[clap(parse(try_from_str))]
    id: PartitionId,

    #[clap(short, long)]
    name: Option<String>,

    // TODO: support generating new unique GUID
    #[clap(short = 'u', long = "guid", help = "Unique GUID")]
    unique_guid: Option<Uuid>,

    #[clap(short = 't', long = "type", parse(try_from_str = parse_partition_type), long_help = "Type GUID or type alias eg. msbasic, msreserved, esp")]
    type_guid: Option<Uuid>,
}

#[derive(Parser)]
pub enum SubCommand {
    #[clap(about = "Create new partition table")]
    Create(CreateOptions),

    #[clap(about = "Print general information about partition table")]
    Info,

    #[clap(about = "Dump raw contents of partition table")]
    Dump,

    #[clap(about = "Add partition")]
    Add(AddOptions),

    #[clap(about = "Delete partition")]
    #[clap(alias = "del")]
    Delete(DeleteOptions),

    #[clap(about = "Modify things like partition name, type, GUID, etc.")]
    #[clap(alias = "mod")]
    Modify(ModifyOptions),
}

#[derive(Parser)]
#[clap(about = "Manipulate GUID partition table")]
pub struct Command {
    #[clap(subcommand)]
    cmd: SubCommand,
}

pub fn run(disk: &CommonDiskOptions, command: Command) -> anyhow::Result<()> {
    let mut disk = open_disk(
        disk.file.as_path(),
        disk.format,
        access::get_access_mode(&command),
    )?;

    if let SubCommand::Create(opt) = command.cmd {
        return create::create(disk.as_mut(), &opt);
    }

    let mut gpt = Gpt::load(disk.as_mut(), ErrorAction::Ignore).context("failed to load GPT")?;

    match command.cmd {
        SubCommand::Create(_) => unreachable!(),
        SubCommand::Dump => dump::dump(disk.as_ref(), &gpt),
        SubCommand::Info => unimplemented!(),
        SubCommand::Add(opt) => add::add(disk.as_mut(), &mut gpt, &opt),
        SubCommand::Delete(opt) => delete::delete(disk.as_mut(), &mut gpt, &opt),
        SubCommand::Modify(opt) => modify::modify(disk.as_mut(), &mut gpt, &opt),
    }
}
