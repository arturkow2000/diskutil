pub mod gpt;
pub mod mbr;

use crate::disk::Disk;
use crate::Result;

pub trait Partition {
    fn start(&self) -> u64;
    fn end(&self) -> u64;
}
pub trait PartitionTable {
    fn get_partition_start_end(&self, index: u32) -> Option<(u64, u64)>;
}

pub fn load_partition_table(disk: &mut dyn Disk) -> Result<Box<dyn PartitionTable>> {
    let gpt = gpt::Gpt::load(disk, gpt::ErrorAction::Abort)?;
    Ok(Box::new(gpt))
}
