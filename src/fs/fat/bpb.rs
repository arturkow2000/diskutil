use crate::{u8_array_uninitialized, Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::fmt;
use std::io::Read;

pub struct BpbFat32 {
    pub jump: [u8; 3],
    pub oem_id: [u8; 8],
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    pub number_of_reserved_sectors: u16,
    pub number_of_fats: u8,
    pub number_of_directory_entries: u16,
    pub sectors_total: u32,
    pub media_descriptor: u8,
    pub sectors_per_fat: u32,
    pub sectors_per_track: u16,
    pub number_of_heads: u16,
    pub number_of_hidden_sectors: u32,
    pub flags: u16,
    pub fat_version: u16,
    pub root_directory_cluster: u32,
    pub fsinfo_lba: u16,
    pub backup_bs_lba: u16,
    pub reserved: [u8; 12],
    pub drive_number: u8,
    pub winnt_flags: u8,
    pub signature: u8,
    pub serial: [u8; 4],
    pub label: String,
    pub identifier: [u8; 8],
    pub boot_code: [u8; 420],
}

impl BpbFat32 {
    pub const SIZE: usize = 512;

    pub fn decode(reader: &mut dyn Read) -> Result<Self> {
        #[cfg(debug_assertions)]
        let mut total_read = 0u64;

        macro_rules! read {
            (array($size:expr)) => {{
                let mut a = u8_array_uninitialized!($size);
                #[cfg(debug_assertions)]
                {
                    total_read += $size as u64;
                }
                reader.read(&mut a)?;
                a
            }};
            (u8) => {{
                let x = reader.read_u8()?;
                #[cfg(debug_assertions)]
                {
                    total_read += 1;
                }
                x
            }};
            (u16) => {{
                let x = reader.read_u16::<LittleEndian>()?;
                #[cfg(debug_assertions)]
                {
                    total_read += 2;
                }
                x
            }};
            (u32) => {{
                let x = reader.read_u32::<LittleEndian>()?;
                #[cfg(debug_assertions)]
                {
                    total_read += 4;
                }
                x
            }};
        }

        let jump = read!(array(3));
        let oem_id = read!(array(8));
        let bytes_per_sector = read!(u16);
        let sectors_per_cluster = read!(u8);
        let number_of_reserved_sectors = read!(u16);
        let number_of_fats = read!(u8);
        let number_of_directory_entries = read!(u16);
        let mut sectors_total = read!(u16) as u32;
        let media_descriptor = read!(u8);
        let _ = read!(u16);
        let sectors_per_track = read!(u16);
        let number_of_heads = read!(u16);
        let number_of_hidden_sectors = read!(u32);
        if sectors_total == 0 {
            sectors_total = read!(u32);
        } else {
            let _ = read!(u32);
        }
        let sectors_per_fat = read!(u32);
        let flags = read!(u16);
        let fat_version = read!(u16);
        let root_directory_cluster = read!(u32);
        let fsinfo_lba = read!(u16);
        let backup_bs_lba = read!(u16);
        let reserved = read!(array(12));
        let drive_number = read!(u8);
        let winnt_flags = read!(u8);
        let signature = read!(u8);
        /*if signature != 0x28 && signature != 0x29 {
            panic!("checkpoint signature: {:x}", signature);
            return Err(Error::InvalidBpb);
        }*/
        let serial = read!(array(4));
        let label = read!(array(11));
        let identifier = read!(array(8));
        /*if &identifier != b"FAT32   " {
            panic!(
                "identifier = [{},{},{},{},{},{},{},{}]",
                identifier[0],
                identifier[1],
                identifier[2],
                identifier[3],
                identifier[4],
                identifier[5],
                identifier[6],
                identifier[7]
            );
            return Err(Error::InvalidBpb);
        }*/
        let boot_code = read!(array(420));
        let bs_signature = read!(u16);
        if bs_signature != 0xAA55 {
            return Err(Error::InvalidBpb);
        }

        debug_assert_eq!(total_read, Self::SIZE as u64);

        Ok(Self {
            jump,
            oem_id,
            bytes_per_sector,
            sectors_per_cluster,
            number_of_reserved_sectors,
            number_of_fats,
            number_of_directory_entries,
            media_descriptor,
            sectors_per_fat,
            number_of_heads,
            number_of_hidden_sectors,
            sectors_total,
            sectors_per_track,
            flags,
            fat_version,
            root_directory_cluster,
            fsinfo_lba,
            backup_bs_lba,
            reserved,
            drive_number,
            winnt_flags,
            signature,
            serial,
            label: String::from_utf8_lossy(&label[..]).to_string(),
            boot_code,
            identifier,
        })
    }
}

impl fmt::Display for BpbFat32 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Oem ID                      : {}
Bytes per sector            : {}
Sectors per cluster         : {}
Reserved sectors            : {}
Number of FATs              : {}
Number of directory entries : {}
Sectors per FAT             : {}
Number of heads             : {}
Number of hidden sectors    : {}
Total sectors               : {}
Label                       : {}",
            String::from_utf8_lossy(&self.oem_id),
            self.bytes_per_sector,
            self.sectors_per_cluster,
            self.number_of_reserved_sectors,
            self.number_of_fats,
            self.number_of_directory_entries,
            self.sectors_per_fat,
            self.number_of_heads,
            self.number_of_hidden_sectors,
            self.sectors_total,
            self.label
        )
    }
}
