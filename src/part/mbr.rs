use crate::disk::{Disk, MediaType};
use crate::part::Partition;
use crate::{Error, Result};
use std::convert::TryInto;
use std::io::{Cursor, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

pub struct MbrPartition {
    pub flags: u8,
    pub start_chs: (u16, u8, u8),
    pub partition_type: u8,
    pub end_chs: (u16, u8, u8),
    pub lba: u32,
    pub num_sectors: u32,
    pub sector_size: u32,
}
impl MbrPartition {
    pub fn decode(sector_size: u32, buf: &[u8; 16]) -> Option<Self> {
        let mut cursor = Cursor::new(buf);

        let flags = cursor.read_u8().unwrap();
        if flags == 0 {
            return None;
        }

        let start_chs = Self::decode_chs(&mut cursor);
        let partition_type = cursor.read_u8().unwrap();
        let end_chs = Self::decode_chs(&mut cursor);

        let lba = cursor.read_u32::<LittleEndian>().unwrap();
        let num_sectors = cursor.read_u32::<LittleEndian>().unwrap();

        debug_assert_eq!(cursor.position(), 16);

        Some(Self {
            flags,
            start_chs,
            partition_type,
            end_chs,
            lba,
            num_sectors,
            sector_size,
        })
    }

    fn decode_chs<T: AsRef<[u8]>>(cursor: &mut Cursor<T>) -> (u16, u8, u8) {
        let x1 = cursor.read_u8().unwrap();
        let x2 = cursor.read_u8().unwrap();
        let x3 = cursor.read_u8().unwrap();

        let head = x1;
        let sector = x2 & 0x3F;
        let cylinder = x3 as u16 | (((x2 as u16) & 0xC0) << 2);

        (cylinder, head, sector)
    }
}

impl Partition for MbrPartition {
    fn start(&self) -> u64 {
        self.lba as u64 * self.sector_size as u64
    }
    fn size(&self) -> u64 {
        self.num_sectors as u64 * self.sector_size as u64
    }
}

pub struct Mbr<'a> {
    #[allow(dead_code)]
    disk: &'a mut dyn Disk,
    pub partitions: Vec<Option<MbrPartition>>,
    pub code: [u8; 446],
}
impl<'a> Mbr<'a> {
    pub fn load(disk: &'a mut dyn Disk) -> Result<Self> {
        let media_type = disk.media_type();
        assert!(media_type == MediaType::HDD || media_type == MediaType::SSD);

        let mut buf = [0u8; 512];

        disk.seek(SeekFrom::Start(0))?;
        disk.read_exact(&mut buf)?;

        if buf[0x1FE] != 0x55 || buf[0x1FF] != 0xAA {
            return Err(Error::MbrMissing);
        }

        let mut partitions: Vec<Option<MbrPartition>> = Vec::with_capacity(4);
        for x in [0x01BE, 0x01CE, 0x01DE, 0x01EE].iter().copied() {
            partitions.push(MbrPartition::decode(
                disk.block_size() as u32,
                &buf[x..x + 0x10].try_into().unwrap(),
            ));
        }

        Ok(Self {
            disk,
            partitions,
            code: buf[..446].try_into().unwrap(),
        })
    }
}
