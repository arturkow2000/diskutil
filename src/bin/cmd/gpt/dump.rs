use std::mem::transmute;

use crate::utils;
use diskutil::disk::Disk;
use diskutil::part::gpt::{uuid128_partition_type_guid_to_name, Gpt};

pub fn dump(disk: &dyn Disk, gpt: &Gpt) -> anyhow::Result<()> {
    println!(
        "{:<5} {:<8} {:<8} {:<8} {:<38} {:<45} Name",
        "Index", "Start", "End", "Size", "Unique GUID", "Type"
    );

    for (i, p) in gpt
        .partitions
        .iter()
        .enumerate()
        .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
    {
        if let Some(size) = p
            .end_lba
            .checked_sub(p.start_lba)
            .map(|x| (x + 1).saturating_mul(disk.sector_size().into()))
        {
            // TODO: replace this with safe alternative
            let t =
                uuid128_partition_type_guid_to_name(unsafe { transmute(p.type_guid.as_u128()) });

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

    Ok(())
}
