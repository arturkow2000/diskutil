mod bpb;

use crate::disk::Disk;
use crate::Result;
use bpb::BpbFat32;
use std::io::{BufReader, Seek, SeekFrom};

pub struct Fat32<'a> {
    #[allow(dead_code)]
    device: &'a mut dyn Disk,
}

impl<'a> Fat32<'a> {
    pub fn open(device: &'a mut dyn Disk) -> Result<Self> {
        let mut reader = BufReader::with_capacity(BpbFat32::SIZE, device);

        reader.seek(SeekFrom::Start(0))?;
        let bpb = BpbFat32::decode(&mut reader)?;
        debug!("{}", bpb);

        todo!()
    }
}
