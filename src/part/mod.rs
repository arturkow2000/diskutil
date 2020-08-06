pub mod gpt;
pub mod mbr;

pub trait Partition {
    fn start(&self) -> u64;
    fn size(&self) -> u64;
}
