mod partition_type;
pub use partition_type::*;

use super::{Partition, PartitionTable};
use crate::disk::Disk;
use crate::utils::{allocate_u8_vector_uninitialized, zero_u8_slice};
use crate::{is_power_of_2, round_up, Error, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::{crc32, Hasher32};
use std::hash::Hasher;
use std::io::{self, BufReader, Cursor, Read, Seek, SeekFrom, Write};
use uuid::Uuid;

const GPT_HEADER_SIZE: usize = 0x5C;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ErrorAction {
    Abort,
    Ignore,
}

pub struct Gpt {
    pub partitions: Vec<Option<GptPartition>>,
    //
    pub revision: u32,
    pub reserved: u32,
    pub current_lba: u64,
    pub alternate_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub disk_guid: Uuid,
    pub partition_table_start: u64,
    pub partition_table_entries_num: u32,
    pub partition_table_entry_size: u32,
    // header_size field omitted here
    // header_size = GPT_HEADER_SIZE + header_additional_data.len()
    pub header_additional_data: Vec<u8>,
}

impl Gpt {
    const ENTRY_SIZE: u32 = 128;
    const REVISION: u32 = 0x00010000;

    // TODO: support loading backup
    pub fn load(disk: &mut dyn Disk, error_action: ErrorAction) -> Result<Self> {
        let sector_size = disk.block_size();
        let mut reader = BufReader::with_capacity(sector_size as usize, disk);
        let mut crc32 = crc32::Digest::new(crc32::IEEE);

        macro_rules! read_hash {
            (u32) => {{
                let t = reader.read_u32::<LittleEndian>();
                if let Ok(t) = t {
                    crc32.write_u32(t);
                }
                t
            }};
            (u32,$x:expr) => {{
                crc32.write_u32($x);
                reader.read_u32::<LittleEndian>()
            }};
            (u64) => {{
                let t = reader.read_u64::<LittleEndian>();
                if let Ok(t) = t {
                    crc32.write_u64(t);
                }
                t
            }};
        }

        let gpt_start_lba = 1u64;
        reader.seek(SeekFrom::Start(gpt_start_lba * sector_size as u64))?;
        let signature = read_hash!(u64)?;
        if signature != 0x5452415020494645u64 {
            return Err(Error::GptMissing);
        }

        let revision = read_hash!(u32)?;
        let header_size = read_hash!(u32)?;
        if header_size < 92 {
            return Err(Error::InvalidGpt(format!(
                "header size ({}) is less than minimum (92)",
                header_size
            )));
        }
        let header_crc32 = read_hash!(u32, 0)?;
        let reserved = read_hash!(u32)?;
        let mut current_lba = read_hash!(u64)?;
        if current_lba != gpt_start_lba {
            current_lba = match error_action {
                ErrorAction::Abort => {
                    return Err(Error::InvalidGpt("Invalid current_lba".to_owned()))
                }
                ErrorAction::Ignore => {
                    warn!(
                        "fixing current_lba from {} to {}",
                        current_lba, gpt_start_lba
                    );
                    gpt_start_lba
                }
            }
        }
        let alternate_lba = read_hash!(u64)?;
        let first_usable_lba = read_hash!(u64)?;
        let last_usable_lba = read_hash!(u64)?;
        let disk_guid = read_guid_hash(&mut reader, &mut crc32)?;
        let partition_table_start = read_hash!(u64)?;
        let partition_table_entries_num = read_hash!(u32)?;
        let partition_table_entry_size = read_hash!(u32)?;
        let partition_table_crc32 = read_hash!(u32)?;

        let partition_table_last_lba = partition_table_start
            + (round_up!(
                partition_table_entry_size as u64 * partition_table_entries_num as u64,
                sector_size as u64
            ) / sector_size as u64)
            - 1;
        if first_usable_lba <= partition_table_last_lba {
            let msg = format!(
                "first_usable_lba overlaps with partition table ({} <= {})",
                first_usable_lba, partition_table_last_lba
            );
            match error_action {
                ErrorAction::Abort => return Err(Error::InvalidGpt(msg)),
                ErrorAction::Ignore => warn!("{}", msg),
            };
        }

        let total_read = reader.seek(SeekFrom::Current(0))? - sector_size as u64;
        assert_eq!(total_read, GPT_HEADER_SIZE as u64);

        let header_additional_data = if total_read != header_size as u64 {
            assert!(total_read < header_size as u64);
            let additional = header_size as u64 - total_read;
            let mut buf = allocate_u8_vector_uninitialized(additional as usize);
            reader.read_exact(buf.as_mut_slice())?;
            Hasher::write(&mut crc32, buf.as_slice());
            buf
        } else {
            Vec::new()
        };

        let computed_crc32 = crc32.sum32();

        debug!("GPT Header dump:");
        debug!("Revision                       : 0x{:08x}", revision);
        debug!("Header Size                    : {}", header_size);
        debug!("Header CRC32                   : 0x{:08x}", header_crc32);
        debug!("Current LBA                    : {}", current_lba);
        debug!("Alternate LBA                  : {}", alternate_lba);
        debug!("First Usable LBA               : {}", first_usable_lba);
        debug!("Last Usable LBA                : {}", last_usable_lba);
        debug!("Disk GUID                      : {{{}}}", disk_guid);
        debug!("Partition Table Start          : {}", partition_table_start);
        debug!(
            "Partition Table Entries Count  : {}",
            partition_table_entries_num
        );
        debug!(
            "Partition Table Entry Size     : {}",
            partition_table_entry_size
        );
        debug!(
            "Partition Table CRC32          : 0x{:08x}",
            partition_table_crc32
        );

        if computed_crc32 != header_crc32 {
            let msg = format!(
                "GPT header checksum mismatch 0x{:08x} (computed) != 0x{:08x}",
                computed_crc32, header_crc32
            );
            match error_action {
                ErrorAction::Abort => return Err(Error::InvalidGpt(msg)),
                ErrorAction::Ignore => warn!("{}", msg),
            };
        }

        let disk = reader.into_inner();
        disk.seek(SeekFrom::Start(
            partition_table_start as u64 * sector_size as u64,
        ))?;
        let mut partition_table_buffer = allocate_u8_vector_uninitialized(
            partition_table_entries_num as usize * partition_table_entry_size as usize,
        );
        disk.read_exact(partition_table_buffer.as_mut_slice())?;

        crc32.reset();
        Hasher::write(&mut crc32, partition_table_buffer.as_slice());

        let computed_crc32 = crc32.sum32();
        if computed_crc32 != partition_table_crc32 {
            let msg = format!(
                "Partition Table checksum mismatch 0x{:08x} (computed) != 0x{:08x}",
                computed_crc32, partition_table_crc32
            );
            match error_action {
                ErrorAction::Abort => return Err(Error::InvalidGpt(msg)),
                ErrorAction::Ignore => warn!("{}", msg),
            };
        }

        let partition_name_len = (partition_table_entry_size as usize - 0x38) / 2;
        let mut partitions: Vec<Option<GptPartition>> =
            Vec::with_capacity(partition_table_entries_num as usize);
        for i in 0..partition_table_entries_num as usize {
            let s = &partition_table_buffer[i * partition_table_entry_size as usize
                ..i * partition_table_entry_size as usize + partition_table_entry_size as usize];
            let mut cursor = Cursor::new(s);

            let type_guid = read_guid(&mut cursor).unwrap();
            let unique_guid = read_guid(&mut cursor).unwrap();
            let start_lba = cursor.read_u64::<LittleEndian>().unwrap();
            let end_lba = cursor.read_u64::<LittleEndian>().unwrap();
            let attributes = cursor.read_u64::<LittleEndian>().unwrap();
            debug_assert_eq!(cursor.position(), 0x38);

            let mut partition_name = String::with_capacity(partition_name_len);

            struct I<'a, T>
            where
                T: AsRef<[u8]>,
            {
                pub cursor: &'a mut Cursor<T>,
                pub left: usize,
            }
            impl<'a, T> Iterator for I<'a, T>
            where
                T: AsRef<[u8]>,
            {
                type Item = u16;
                fn next(&mut self) -> Option<u16> {
                    if self.left > 0 {
                        self.left -= 1;
                        let x = self.cursor.read_u16::<LittleEndian>().unwrap();
                        if x == 0 {
                            None
                        } else {
                            Some(x)
                        }
                    } else {
                        None
                    }
                }
            }

            for x in ::std::char::decode_utf16(I {
                cursor: &mut cursor,
                left: partition_name_len,
            }) {
                partition_name.push(x.unwrap_or(::std::char::REPLACEMENT_CHARACTER));
            }

            let is_null_entry = type_guid.is_nil()
                && unique_guid.is_nil()
                && start_lba == 0
                && end_lba == 0
                && attributes == 0
                && partition_name.is_empty();

            if !is_null_entry {
                debug!(
                    "{:>5} 0x{:016x} 0x{:016x} {{{}}} {{{}}} 0x{:016x} {}",
                    i, start_lba, end_lba, type_guid, unique_guid, attributes, partition_name
                );
                partitions.push(Some(GptPartition {
                    type_guid,
                    unique_guid,
                    start_lba,
                    end_lba,
                    attributes,
                    partition_name,
                }));
            } else {
                partitions.push(None)
            }
        }

        Ok(Self {
            partitions,
            revision,
            reserved,
            current_lba,
            alternate_lba,
            first_usable_lba,
            last_usable_lba,
            disk_guid,
            partition_table_start,
            partition_table_entries_num,
            partition_table_entry_size,
            header_additional_data,
        })
    }

    pub fn create(disk: &mut dyn Disk) -> Result<Self> {
        Self::create_ex(disk, 128)
    }
    pub fn create_ex(disk: &mut dyn Disk, max_entries: u32) -> Result<Self> {
        let sector_size = disk.block_size();

        let partition_table_start = 2;
        let partition_table_entries_num = max_entries;
        let partition_table_entry_size = Self::ENTRY_SIZE;

        let partition_table_size_in_sectors = round_up!(
            partition_table_entry_size as u64 * partition_table_entries_num as u64,
            sector_size as u64
        ) / sector_size as u64;
        let first_usable_lba = partition_table_start + partition_table_size_in_sectors;

        let disk_size = disk.max_disk_size();
        assert_eq!(disk_size % sector_size as u64, 0);
        let alternate_lba = disk_size / sector_size as u64;
        let last_usable_lba = alternate_lba - partition_table_size_in_sectors - 1;

        Ok(Self {
            partitions: Vec::new(),
            revision: Self::REVISION,
            reserved: 0,
            current_lba: 1,
            alternate_lba,
            first_usable_lba,
            last_usable_lba,
            disk_guid: Uuid::new_v4(),
            partition_table_start,
            partition_table_entries_num,
            partition_table_entry_size,
            header_additional_data: Vec::new(),
        })
    }

    pub fn update(&mut self, disk: &mut dyn Disk) -> Result<()> {
        let sector_size = disk.block_size();
        let mut cursor = Cursor::new(allocate_u8_vector_uninitialized(round_up!(
            self.partition_table_entries_num as usize * self.partition_table_entry_size as usize,
            sector_size as usize
        )));
        let mut crc32 = crc32::Digest::new(crc32::IEEE);

        macro_rules! write_hash {
            (u64, $cursor:tt, $x:expr) => {{
                crc32.write_u64($x);
                $cursor.write_u64::<LittleEndian>($x)
            }};
            (u32, $cursor:tt, $x:expr) => {{
                crc32.write_u32($x);
                $cursor.write_u32::<LittleEndian>($x)
            }};
            (u16, $cursor:tt, $x:expr) => {{
                crc32.write_u16($x);
                $cursor.write_u16::<LittleEndian>($x)
            }};
        }

        for i in 0..self.partition_table_entries_num as usize {
            if let Some(partition) = self.partitions.get(i).and_then(|x| x.as_ref()) {
                let start_position = cursor.position();

                write_guid_hash(&mut cursor, &mut crc32, partition.type_guid).unwrap();
                write_guid_hash(&mut cursor, &mut crc32, partition.unique_guid).unwrap();
                write_hash!(u64, cursor, partition.start_lba).unwrap();
                write_hash!(u64, cursor, partition.end_lba).unwrap();
                write_hash!(u64, cursor, partition.attributes).unwrap();

                debug_assert_eq!(cursor.position() - 0x38, start_position);

                assert!(
                    partition.partition_name.len()
                        <= self.partition_table_entry_size as usize - 0x38
                );

                let mut n = 0x38;
                for x in partition.partition_name.encode_utf16() {
                    write_hash!(u16, cursor, x).unwrap();
                    n += 2;
                }

                debug_assert!(n <= self.partition_table_entry_size);

                if n != self.partition_table_entry_size {
                    let s = &mut cursor.get_mut()[start_position as usize + n as usize
                        ..start_position as usize + self.partition_table_entry_size as usize];
                    assert_eq!(
                        s.len(),
                        self.partition_table_entry_size as usize - n as usize
                    );
                    zero_u8_slice(s);
                    Hasher::write(&mut crc32, s);
                }
            } else {
                let position = cursor.position();
                let s = &mut cursor.get_mut()[position as usize
                    ..position as usize + self.partition_table_entry_size as usize];
                zero_u8_slice(s);
                Hasher::write(&mut crc32, s);
                cursor
                    .seek(SeekFrom::Current(self.partition_table_entry_size as i64))
                    .unwrap();
            }
        }

        let partition_table_buffer = cursor.into_inner();
        let partition_table_crc32 = crc32.sum32();

        crc32.reset();
        assert!(self.header_additional_data.len() + GPT_HEADER_SIZE <= sector_size as usize);
        let mut cursor = Cursor::new(allocate_u8_vector_uninitialized(sector_size as usize));

        write_hash!(u64, cursor, 0x5452415020494645u64).unwrap();
        write_hash!(u32, cursor, self.revision).unwrap();
        write_hash!(
            u32,
            cursor,
            GPT_HEADER_SIZE as u32 + self.header_additional_data.len() as u32
        )
        .unwrap();
        write_hash!(u32, cursor, 0).unwrap();
        write_hash!(u32, cursor, self.reserved).unwrap();
        write_hash!(u64, cursor, self.current_lba).unwrap();
        write_hash!(u64, cursor, self.alternate_lba).unwrap();
        write_hash!(u64, cursor, self.first_usable_lba).unwrap();
        write_hash!(u64, cursor, self.last_usable_lba).unwrap();
        write_guid_hash(&mut cursor, &mut crc32, self.disk_guid).unwrap();
        write_hash!(u64, cursor, self.partition_table_start).unwrap();
        write_hash!(u32, cursor, self.partition_table_entries_num).unwrap();
        write_hash!(u32, cursor, self.partition_table_entry_size).unwrap();
        write_hash!(u32, cursor, partition_table_crc32).unwrap();

        debug_assert_eq!(cursor.position(), GPT_HEADER_SIZE as u64);
        if !self.header_additional_data.is_empty() {
            cursor
                .write_all(self.header_additional_data.as_slice())
                .unwrap();
            Hasher::write(&mut crc32, self.header_additional_data.as_slice());
        }
        let p = cursor.position();
        debug_assert!(p <= sector_size as u64);
        cursor.set_position(16);
        cursor.write_u32::<LittleEndian>(crc32.sum32()).unwrap();

        if p != sector_size as u64 {
            zero_u8_slice(&mut cursor.get_mut()[p as usize..sector_size as usize]);
        }

        let header = cursor.into_inner();

        // TODO: check if contiguous and use write_all_vectored if possible
        disk.seek(SeekFrom::Start(self.current_lba * sector_size as u64))?;
        disk.write_all(header.as_slice())?;
        disk.seek(SeekFrom::Start(
            self.partition_table_start * sector_size as u64,
        ))?;
        disk.write_all(partition_table_buffer.as_slice())?;

        // Write backup partition table
        let partition_table_size_in_sectors = round_up!(
            self.partition_table_entry_size as u64 * self.partition_table_entries_num as u64,
            sector_size as u64
        ) / sector_size as u64;
        let backup_partition_table_lba = self.alternate_lba - partition_table_size_in_sectors;
        disk.seek(SeekFrom::Start(
            backup_partition_table_lba * sector_size as u64,
        ))?;
        disk.write_all(partition_table_buffer.as_slice())?;

        // Write backup header
        crc32.reset();
        let mut cursor = Cursor::new(allocate_u8_vector_uninitialized(sector_size as usize));
        write_hash!(u64, cursor, 0x5452415020494645u64).unwrap();
        write_hash!(u32, cursor, self.revision).unwrap();
        write_hash!(
            u32,
            cursor,
            GPT_HEADER_SIZE as u32 + self.header_additional_data.len() as u32
        )
        .unwrap();
        write_hash!(u32, cursor, 0).unwrap();
        write_hash!(u32, cursor, self.reserved).unwrap();
        write_hash!(u64, cursor, self.alternate_lba).unwrap();
        write_hash!(u64, cursor, self.current_lba).unwrap();
        write_hash!(u64, cursor, self.first_usable_lba).unwrap();
        write_hash!(u64, cursor, self.last_usable_lba).unwrap();
        write_guid_hash(&mut cursor, &mut crc32, self.disk_guid).unwrap();
        write_hash!(u64, cursor, backup_partition_table_lba).unwrap();
        write_hash!(u32, cursor, self.partition_table_entries_num).unwrap();
        write_hash!(u32, cursor, self.partition_table_entry_size).unwrap();
        write_hash!(u32, cursor, partition_table_crc32).unwrap();

        debug_assert_eq!(cursor.position(), GPT_HEADER_SIZE as u64);
        if !self.header_additional_data.is_empty() {
            cursor
                .write_all(self.header_additional_data.as_slice())
                .unwrap();
            Hasher::write(&mut crc32, self.header_additional_data.as_slice());
        }
        let p = cursor.position();
        debug_assert!(p <= sector_size as u64);
        cursor.set_position(16);
        cursor.write_u32::<LittleEndian>(crc32.sum32()).unwrap();

        let header = cursor.into_inner();
        disk.seek(SeekFrom::Start(self.alternate_lba * sector_size as u64))?;
        disk.write_all(header.as_slice())?;

        Ok(())
    }
}

impl PartitionTable for Gpt {
    fn get_partition_start_end(&self, index: u32) -> Option<(u64, u64)> {
        if let Some(x) = self.partitions.get(index as usize) {
            if let Some(x) = x {
                Some((x.start_lba, x.end_lba))
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub struct GptPartition {
    pub type_guid: Uuid,
    pub unique_guid: Uuid,
    pub start_lba: u64,
    pub end_lba: u64,
    pub attributes: u64,
    pub partition_name: String,
}

impl GptPartition {
    pub fn new(_type: GptPartitionType, name: &str, start: u64, end: u64) -> Self {
        Self::new_with_type_guid(_type.to_guid(), name, start, end)
    }

    pub fn new_with_type_guid(type_guid: Uuid, name: &str, start: u64, end: u64) -> Self {
        Self {
            type_guid,
            unique_guid: Uuid::new_v4(),
            start_lba: start,
            end_lba: end,
            attributes: 0,
            partition_name: name.to_owned(),
        }
    }
}

impl Partition for GptPartition {
    fn start(&self) -> u64 {
        self.start_lba
    }
    fn end(&self) -> u64 {
        self.end_lba
    }
}

fn read_guid_hash<T, H>(reader: &mut T, hasher: &mut H) -> Result<Uuid>
where
    T: Read,
    H: Hasher32 + Hasher,
{
    let p0 = reader.read_u32::<LittleEndian>()?;
    let p1 = reader.read_u16::<LittleEndian>()?;
    let p2 = reader.read_u16::<LittleEndian>()?;
    let mut p3 = [0u8; 8];
    reader.read_exact(&mut p3)?;

    hasher.write_u32(p0);
    hasher.write_u16(p1);
    hasher.write_u16(p2);
    Hasher::write(hasher, &p3[..]);

    Ok(Uuid::from_fields(p0, p1, p2, &p3).unwrap())
}

fn read_guid<T>(reader: &mut T) -> Result<Uuid>
where
    T: Read,
{
    let p0 = reader.read_u32::<LittleEndian>()?;
    let p1 = reader.read_u16::<LittleEndian>()?;
    let p2 = reader.read_u16::<LittleEndian>()?;
    let mut p3 = [0u8; 8];
    reader.read_exact(&mut p3)?;

    Ok(Uuid::from_fields(p0, p1, p2, &p3).unwrap())
}

pub fn write_guid_hash<T, H>(writer: &mut T, hasher: &mut H, uuid: Uuid) -> io::Result<()>
where
    T: Write,
    H: Hasher32 + Hasher,
{
    let (p0, p1, p2, p3) = uuid.as_fields();

    writer.write_u32::<LittleEndian>(p0)?;
    writer.write_u16::<LittleEndian>(p1)?;
    writer.write_u16::<LittleEndian>(p2)?;
    writer.write_all(&p3[..])?;

    hasher.write_u32(p0);
    hasher.write_u16(p1);
    hasher.write_u16(p2);
    Hasher::write(hasher, &p3[..]);

    Ok(())
}
