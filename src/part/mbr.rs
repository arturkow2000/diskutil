use std::convert::TryInto;
use std::io::{self, Cursor, SeekFrom, Write};

use super::PartitionTable;
use crate::disk::{Disk, MediaType};
use crate::{u8_array_uninitialized, Error, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[rustfmt::skip]
pub const CODE_NONBOOTABLE: [u8; 446] = 
[
    0x31, 0xc0, 0x8e, 0xd8, 0x8e, 0xd0, 0xb8, 0x00, 0x7c, 0x89, 0xc4, 0xbe, 0x2d, 0x7c, // org 0x7c00
    0xe8, 0x07, 0x00, 0xcd, 0x18, 0xfa, 0xf4, 0xe9, 0xfc, 0xff, 0xb4, 0x0e, 0xbb, 0x1f, // xor ax, ax
    0x00, 0xb9, 0x01, 0x00, 0xac, 0x84, 0xc0, 0x74, 0x07, 0x56, 0xcd, 0x10, 0x5e, 0xe9, // mov ds, ax
    0xec, 0xff, 0xc3, 0x4e, 0x6f, 0x74, 0x20, 0x61, 0x20, 0x62, 0x6f, 0x6f, 0x74, 0x61, // mov ss, ax
    0x62, 0x6c, 0x65, 0x20, 0x64, 0x69, 0x73, 0x6b, 0x2e, 0x0a, 0x0d, 0x00, 0x00, 0x00, // mov ax, 0x7c00
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov sp, ax
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov si, message
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // call print
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // int 18h
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // cli
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // halt:
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // hlt
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // jmp halt
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // print:
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov ah, 0eh
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov bx, 001fh
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov cx, 1
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // lodsb
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // test al, al
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // jz .e
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // push si
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // int 10h
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // pop si
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // jmp print
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // .e:
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // ret
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // message: db "Not a bootable disk.", 10, 13, 0
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // times 446 - ($ - $$) db 0
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[allow(dead_code)]
fn chs_to_lba(chs: (u16, u8, u8), heads_per_cylinder: u32, sectors_per_track: u32) -> u32 {
    (chs.0 as u32 * heads_per_cylinder + chs.1 as u32) * sectors_per_track + (chs.2 as u32 - 1)
}

#[allow(dead_code)]
fn lba_to_chs(lba: u32, heads_per_cylinder: u32, sectors_per_track: u32) -> (u16, u8, u8) {
    let c = lba / (heads_per_cylinder * sectors_per_track);
    let h = (lba / sectors_per_track) % heads_per_cylinder;
    let s = (lba % sectors_per_track) + 1;
    (
        c.try_into().unwrap(),
        h.try_into().unwrap(),
        s.try_into().unwrap(),
    )
}

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
        let start_chs = Self::decode_chs(&mut cursor);
        let partition_type = cursor.read_u8().unwrap();
        let end_chs = Self::decode_chs(&mut cursor);

        let lba = cursor.read_u32::<LittleEndian>().unwrap();
        let num_sectors = cursor.read_u32::<LittleEndian>().unwrap();

        if num_sectors == 0 {
            return None;
        }

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

    /// Returns LBA of the first sector belonging to partition.
    pub fn start(&self) -> u32 {
        // TODO: should verify whether LBA is valid
        // Some MBRs (older) may use only CHS addressing. CHS used to depend on
        // underlying disk parameters, since IDE drives came out, CHS is
        // emulated. To implement CHS support we need support in disk layer for
        // querying disk geometry.
        //
        // In case of physial disks OS provides geometry, in case of virtual
        // disks specific disk format may or may not have geometry. In case of
        // raw disks there is no way to query geometry except by assuming some
        // defaults (which user could override if needed). Also geometry could
        // be guessed by brute forcing all possible HPC/SPT combinations and
        // comparing results against LBA from partition entries.

        self.lba
    }

    /// Returns LBA of the last sector belonging to partition.
    pub fn end(&self) -> u32 {
        // TODO: CHS support, see the comment above

        self.lba + self.num_sectors - 1
    }

    /// Returns partition size in sectors.
    pub fn size(&self) -> u32 {
        self.num_sectors
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

    fn encode_chs(chs: (u16, u8, u8), writer: &mut dyn Write) -> io::Result<()> {
        let x1 = chs.1;
        let x2 = chs.2 & 0x3F | ((chs.0 & 0x300) >> 2) as u8;
        let x3 = (chs.0 & 0xFF) as u8;

        writer.write_u8(x1)?;
        writer.write_u8(x2)?;
        writer.write_u8(x3)?;

        Ok(())
    }

    fn write(&self, writer: &mut dyn Write) -> io::Result<()> {
        writer.write_u8(self.flags)?;

        Self::encode_chs(self.start_chs, writer)?;
        writer.write_u8(self.partition_type)?;
        Self::encode_chs(self.end_chs, writer)?;

        writer.write_u32::<LittleEndian>(self.lba).unwrap();
        writer.write_u32::<LittleEndian>(self.num_sectors).unwrap();

        Ok(())
    }
}

pub struct Mbr {
    pub partitions: [Option<MbrPartition>; 4],
    pub code: [u8; 446],
}
impl Mbr {
    pub fn load(disk: &mut dyn Disk) -> Result<Self> {
        let media_type = disk.media_type();
        assert!(media_type == MediaType::HDD || media_type == MediaType::SSD);

        let mut buf = u8_array_uninitialized!(512);

        disk.seek(SeekFrom::Start(0))?;
        disk.read_exact(&mut buf)?;

        if buf[0x1FE] != 0x55 || buf[0x1FF] != 0xAA {
            return Err(Error::MbrMissing);
        }

        let mut partitions: [Option<MbrPartition>; 4] = [None, None, None, None];
        for (i, x) in [0x01BE, 0x01CE, 0x01DE, 0x01EE].iter().copied().enumerate() {
            partitions[i] = MbrPartition::decode(
                disk.sector_size() as u32,
                &buf[x..x + 0x10].try_into().unwrap(),
            );
        }

        Ok(Self {
            partitions,
            code: buf[..446].try_into().unwrap(),
        })
    }

    pub fn update(&mut self, disk: &mut dyn Disk) -> Result<()> {
        let mut buffer = u8_array_uninitialized!(512);
        let mut cursor = Cursor::new(&mut buffer[..]);
        cursor.write_all(&self.code).unwrap();
        for p in self.partitions.iter() {
            if let Some(p) = p {
                p.write(&mut cursor).unwrap();
            } else {
                let z = [0u8; 16];
                cursor.write_all(&z).unwrap();
            }
        }

        cursor.write_all(&[0x55, 0xaa]).unwrap();

        debug_assert_eq!(cursor.position(), 512);

        disk.seek(SeekFrom::Start(0))?;
        disk.write_all(&buffer[..])?;

        Ok(())
    }

    pub fn create_protective(disk: &mut dyn Disk) -> Self {
        let sector_size = disk.sector_size();
        let size = disk.disk_size();
        let num_sectors = size / sector_size as u64;
        Self {
            partitions: [
                Some(MbrPartition {
                    flags: 0,
                    start_chs: (0, 0, 2),
                    end_chs: (1023, 255, 63),
                    partition_type: 0xEE,
                    lba: 1,
                    num_sectors: (num_sectors - 1).try_into().unwrap_or(u32::MAX),
                    sector_size,
                }),
                None,
                None,
                None,
            ],
            code: CODE_NONBOOTABLE,
        }
    }
}

impl PartitionTable for Mbr {
    fn get_partition_start_end(&self, index: u32) -> Option<(u64, u64)> {
        if let Some(Some(part)) = self.partitions.get(index as usize) {
            Some((part.lba as u64, part.lba as u64 + part.num_sectors as u64))
        } else {
            None
        }
    }

    fn find_partition_by_guid(&self, _guid: uuid::Uuid) -> Result<(u32, &dyn super::Partition)> {
        // MBR has no GUIDs
        Err(Error::NotSupported)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::MbrPartition;

    fn decode_chs(chs: [u8; 3]) -> (u16, u8, u8) {
        let mut cursor = Cursor::new(&chs[..]);
        MbrPartition::decode_chs(&mut cursor)
    }

    fn encode_chs(chs: (u16, u8, u8)) -> [u8; 3] {
        let mut data = [0u8; 3];
        let mut cursor = Cursor::new(&mut data[..]);
        MbrPartition::encode_chs(chs, &mut cursor).unwrap();
        data
    }

    fn chs_to_lba(chs: (u16, u8, u8)) -> u32 {
        super::chs_to_lba(chs, 16, 63)
    }

    fn lba_to_chs(lba: u32) -> (u16, u8, u8) {
        super::lba_to_chs(lba, 16, 63)
    }

    #[test]
    fn test_decode_chs() {
        assert_eq!(decode_chs([0x00, 0x20, 0x21]), (33, 0, 32));
        assert_eq!(decode_chs([0x00, 0x02, 0x00]), (0, 0, 2));
        assert_eq!(decode_chs([0xee, 0xff, 0xff]), (1023, 238, 63));
        assert_eq!(decode_chs([0xff, 0xff, 0xff]), (1023, 255, 63));
        assert_eq!(decode_chs([0x14, 0x10, 0x04]), (4, 20, 16));
    }

    #[test]
    fn test_encode_chs() {
        assert_eq!(encode_chs((33, 0, 32)), [0x00, 0x20, 0x21]);
        assert_eq!(encode_chs((0, 0, 2)), [0x00, 0x02, 0x00]);
        assert_eq!(encode_chs((1023, 238, 63)), [0xee, 0xff, 0xff]);
        assert_eq!(encode_chs((1023, 255, 63)), [0xff, 0xff, 0xff]);
        assert_eq!(encode_chs((4, 20, 16)), [0x14, 0x10, 0x04]);
    }

    #[test]
    fn test_chs_to_lba() {
        assert_eq!(chs_to_lba((32, 0, 1)), 32256);
        assert_eq!(chs_to_lba((31, 15, 63)), 32255);
        assert_eq!(chs_to_lba((2, 0, 1)), 2016);
        assert_eq!(chs_to_lba((1, 1, 63)), 1133);
    }

    #[test]
    fn test_lba_to_chs() {
        assert_eq!(lba_to_chs(32256), (32, 0, 1));
        assert_eq!(lba_to_chs(32255), (31, 15, 63));
        assert_eq!(lba_to_chs(2016), (2, 0, 1));
        assert_eq!(lba_to_chs(1133), (1, 1, 63));
    }
}
