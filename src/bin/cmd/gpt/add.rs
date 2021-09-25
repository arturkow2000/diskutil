use super::AddOptions;
use anyhow::Context;
use diskutil::disk::Disk;
use diskutil::part::gpt::{Gpt, GptPartition, GptPartitionType};
use diskutil::region::Region;
use uuid::Uuid;

fn find_free_region(gpt: &Gpt, size: u64) -> Option<Region<u64>> {
    let regions = gpt.find_free_regions();
    for region in regions.iter() {
        if region.size() >= size {
            return Some(*region);
        }
    }

    None
}

pub fn add(disk: &mut dyn Disk, gpt: &mut Gpt, opt: &AddOptions) -> anyhow::Result<()> {
    let sector_size = disk.sector_size() as u64;

    if opt.size % sector_size != 0 {
        bail!("partition size is not multiple of sector size")
    }

    let usable_region = Region::new(gpt.first_usable_lba, gpt.last_usable_lba);
    let new_part_region = if let Some(start) = opt.start {
        let region = Region::new_with_size(start, opt.size / sector_size);

        if !region.belongs(&usable_region) {
            bail!("new partition does not fit into usable region")
        }

        for (i, p) in gpt
            .partitions
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
        {
            let used_region = Region::new(p.start_lba, p.end_lba);
            if region.overlaps(&used_region) {
                bail!("new partition would overlap with #{}", i);
            }
        }

        region
    } else {
        let free = find_free_region(gpt, opt.size / sector_size)
            .ok_or_else(|| anyhow::Error::msg("not enough free space"))?;

        Region::new_with_size(free.start(), opt.size / sector_size)
    };

    if let Some(free_slot) = gpt.partitions.iter().position(|x| x.is_none()) {
        gpt.partitions[free_slot] = Some(GptPartition::new_ex(
            opt.type_guid
                .unwrap_or_else(|| GptPartitionType::MicrosoftBasicData.to_guid()),
            opt.name.as_deref().unwrap_or(""),
            new_part_region.start(),
            new_part_region.end(),
            opt.unique_guid.unwrap_or_else(Uuid::new_v4),
        ))
    } else {
        bail!("partition table is full")
    }

    gpt.update(disk).context("failed to update GPT")
}
