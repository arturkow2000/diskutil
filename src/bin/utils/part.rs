use super::PartitionId;
use anyhow::Context;
use diskutil::part::PartitionTable;
use diskutil::region::Region;

pub fn get_partition_region(
    pt: &dyn PartitionTable,
    part: &PartitionId,
) -> anyhow::Result<Region<u64>> {
    let (start, end) = match part {
        PartitionId::Guid(guid) => pt
            .find_partition_by_guid(*guid)
            .map(|(_, x)| (x.start(), x.end()))
            .context("partition not found")?,
        PartitionId::Index(index) => pt
            .get_partition_start_end(*index)
            .ok_or_else(|| anyhow::Error::msg("partition not found"))?,
    };

    Ok(Region::new(start, end))
}
