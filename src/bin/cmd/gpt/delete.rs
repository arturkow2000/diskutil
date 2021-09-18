use anyhow::Context;

use super::DeleteOptions;
use crate::utils::PartitionId;
use diskutil::disk::Disk;
use diskutil::part::gpt::Gpt;

pub fn delete(disk: &mut dyn Disk, gpt: &mut Gpt, opt: &DeleteOptions) -> anyhow::Result<()> {
    let index = match opt.id {
        PartitionId::Index(i) => {
            if gpt.get_partition(i).is_none() {
                bail!("no such partition");
            }

            i
        }
        PartitionId::Guid(g) => {
            if let Ok((i, _)) = gpt.find_partition_by_guid(g) {
                i
            } else {
                bail!("no such partition")
            }
        }
    };

    gpt.partitions[index as usize] = None;

    gpt.update(disk).context("failed to update GPT")
}
