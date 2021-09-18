use super::ModifyOptions;
use crate::utils::PartitionId;
use anyhow::Context;
use diskutil::disk::Disk;
use diskutil::part::gpt::Gpt;

pub fn modify(disk: &mut dyn Disk, gpt: &mut Gpt, opt: &ModifyOptions) -> anyhow::Result<()> {
    let part = match opt.id {
        PartitionId::Index(i) => gpt
            .get_partition_mut(i)
            .ok_or(anyhow::Error::msg("no such partition"))?,
        PartitionId::Guid(g) => {
            if let Ok(part) = gpt.find_partition_by_guid_mut(g).map(|(_, x)| x) {
                part
            } else {
                bail!("no such partition")
            }
        }
    };

    if let Some(name) = opt.name.as_deref() {
        part.partition_name = name.to_owned();
    }

    if let Some(unique_guid) = opt.unique_guid {
        part.unique_guid = unique_guid;
    }

    if let Some(type_guid) = opt.type_guid {
        part.type_guid = type_guid;
    }

    gpt.update(disk).context("failed to update GPT")
}
