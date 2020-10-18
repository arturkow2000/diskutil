use crate::disk::vhd::{dynamic_header::DynamicHeader, footer::Footer, DiskType as VhdDiskType};
use crate::disk::{ArgumentMap, Backend, Disk, DiskFormat, MediaType};
use crate::{is_power_of_2, round_up, u8_array_uninitialized, utils::zero_u8_slice, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::cmp::min;
use std::convert::TryInto;
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::slice;

const SECTOR_SIZE: u32 = 512;

pub struct VhdDisk {
    backend: Box<dyn Backend>,
    footer: Footer,
    // cache encoded footer so we don't have to re-encode it on every rewrite
    footer_encoded: [u8; Footer::SIZE],
    footer_encoded_valid: bool,
    dynamic_header: DynamicHeader,
    bat: Vec<u32>,

    block_size: u32,
    bitmap_size: u32,
    max_disk_size: usize,

    cursor: u64,
    free_data_block_offset: u64,
}

impl VhdDisk {
    pub fn open_with_argmap(backend: Box<dyn Backend>, _args: &ArgumentMap) -> Result<Self> {
        Self::open(backend)
    }

    pub fn open(mut backend: Box<dyn Backend>) -> Result<Self> {
        let _file_size = backend.seek(SeekFrom::End(
            -TryInto::<i64>::try_into(Footer::SIZE).unwrap(),
        ))? as usize
            + Footer::SIZE;
        let mut reader = BufReader::with_capacity(65536, backend);
        let mut footer_encoded = u8_array_uninitialized!(Footer::SIZE);
        reader.read_exact(&mut footer_encoded)?;
        let footer = Footer::decode(&footer_encoded)?;
        // TODO: try footer backup in case of failure
        debug!("Footer:\n{}\n", footer);

        // TODO: support other disk types
        assert_eq!(footer.disk_type, VhdDiskType::Dynamic);
        reader.seek(SeekFrom::Start(footer.data_offset))?;
        let mut dynamic_header_encoded = u8_array_uninitialized!(DynamicHeader::SIZE);
        reader.read_exact(&mut dynamic_header_encoded[..])?;

        let dynamic_header = DynamicHeader::decode(&dynamic_header_encoded)?;
        debug!("Dynamic header:\n{}", dynamic_header);

        let block_size = dynamic_header.block_size;
        let bitmap_size = ((block_size / (8 * 512)) + 511) & !511;
        debug!("Bitmap size : {}", bitmap_size);

        let mut bat = Vec::with_capacity(dynamic_header.max_table_entries as usize);
        reader.seek(SeekFrom::Start(dynamic_header.bat_offset))?;
        let mut free_data_block_offset = round_up!(
            dynamic_header.bat_offset + dynamic_header.max_table_entries as u64 * 4,
            SECTOR_SIZE as u64
        );
        for i in 0..dynamic_header.max_table_entries as usize {
            let entry = reader.read_u32::<BigEndian>()?;
            bat.push(entry);
            if entry != 0xFFFFFFFF {
                let next =
                    entry as u64 * SECTOR_SIZE as u64 + bitmap_size as u64 + block_size as u64;
                if next > free_data_block_offset {
                    free_data_block_offset = next;
                }

                let offset = entry as u64 * SECTOR_SIZE as u64 + bitmap_size as u64;
                // TODO: check if regions overlap
                trace!(
                    "BAT#{:<8} => {:#x}   {{{:#x} - {:#x}}}",
                    i,
                    entry,
                    offset,
                    offset + block_size as u64 - 1
                );
            }
        }

        // FIXME: use chs for size calculation
        let max_disk_size =
            dynamic_header.max_table_entries as usize * dynamic_header.block_size as usize;
        trace!("max_disk_size = {}", max_disk_size);

        Ok(Self {
            backend: reader.into_inner(),
            footer,
            footer_encoded,
            footer_encoded_valid: true,
            dynamic_header,
            bat,
            block_size,
            bitmap_size,
            max_disk_size,
            cursor: 0,
            free_data_block_offset,
        })
    }

    fn get_offset(&mut self, offset: u64, write: bool) -> io::Result<Option<Option<u64>>> {
        let bat_index = offset as usize / self.block_size as usize;
        let offset_in_block = offset % self.block_size as u64;

        if let Some(e) = self.bat.get(bat_index).copied() {
            if e == 0xFFFFFFFF {
                Ok(Some(None))
            } else {
                if write {
                    self.backend
                        .seek(SeekFrom::Start(e as u64 * SECTOR_SIZE as u64))?;
                    let mut bitmap: Vec<u8> = Vec::new();
                    bitmap.resize_with(self.bitmap_size as usize, || 0xff);
                    self.backend.write_all(bitmap.as_slice())?;
                }

                Ok(Some(Some(
                    e as u64 * SECTOR_SIZE as u64 + self.bitmap_size as u64 + offset_in_block,
                )))
            }
        } else {
            Ok(None)
        }
    }

    pub fn create_dynamic(backend: Box<dyn Backend>, max_disk_size: usize) -> io::Result<Self> {
        Self::create_dynamic_ex(backend, max_disk_size, 1024 * 1024 * 2)
    }
    pub fn create_dynamic_ex(
        mut backend: Box<dyn Backend>,
        max_disk_size: usize,
        block_size: usize,
    ) -> io::Result<Self> {
        let max_sectors = {
            let mut t = max_disk_size / 512;
            if max_disk_size % 512 != 0 {
                t += 1;
            }
            t
        };

        // TODO: verify max_sectors and block_size

        let footer = Footer::create(VhdDiskType::Dynamic, max_sectors);
        let mut footer_encoded = u8_array_uninitialized!(Footer::SIZE);
        footer.encode(&mut footer_encoded);
        backend.write_all(&footer_encoded)?;

        let mut bat_size = max_sectors / (block_size / SECTOR_SIZE as usize);
        if max_sectors % (block_size / SECTOR_SIZE as usize) != 0 {
            bat_size += 1;
        }

        let dynamic_header = DynamicHeader::create_dynamic(bat_size, block_size);
        let mut dynamic_header_encoded = u8_array_uninitialized!(DynamicHeader::SIZE);
        dynamic_header.encode(&mut dynamic_header_encoded);
        backend.write_all(&dynamic_header_encoded)?;

        let mut bat: Vec<u32> = Vec::new();
        bat.resize_with(bat_size, || 0xFFFFFFFF);

        unsafe {
            let bat_u8s: &[u8] = slice::from_raw_parts(bat.as_ptr() as *const u8, bat.len() * 4);
            backend.write_all(bat_u8s)?;

            if bat_u8s.len() % SECTOR_SIZE as usize != 0 {
                let padding_len = ((bat_u8s.len() + 511) & !511) - bat_u8s.len();
                let mut padding: Vec<u8> = Vec::new();
                padding.resize_with(padding_len, || 0xff);
                backend.write_all(padding.as_slice())?;
            }
        }

        backend.write_all(&footer_encoded)?;

        let free_data_block_offset = round_up!(
            dynamic_header.bat_offset + bat.len() as u64 * 4,
            SECTOR_SIZE as u64
        );

        Ok(Self {
            backend,
            footer,
            footer_encoded,
            footer_encoded_valid: true,
            dynamic_header,
            bat,
            block_size: block_size as u32,
            bitmap_size: ((block_size as u32 / (8 * 512)) + 511) & !511,
            max_disk_size: max_sectors * 512,
            cursor: 0,
            free_data_block_offset,
        })
    }

    fn alloc_block(&mut self, offset: u64) -> io::Result<u64> {
        let bat_index = offset as usize / self.block_size as usize;
        assert_eq!(self.bat[bat_index], 0xFFFFFFFF);

        let bat_value = (self.free_data_block_offset / SECTOR_SIZE as u64) as u32;
        self.bat[bat_index] = bat_value;

        debug!("allocating block => BAT#{} = {}", bat_index, bat_value);

        let mut bitmap: Vec<u8> = Vec::new();
        bitmap.resize_with(self.bitmap_size as usize, || 0xff);
        self.backend
            .seek(SeekFrom::Start(self.free_data_block_offset))?;
        self.backend.write_all(bitmap.as_slice())?;

        let next_offset =
            self.free_data_block_offset + self.block_size as u64 + self.bitmap_size as u64;
        self.backend.seek(SeekFrom::Start(next_offset))?;
        self.rewrite_footer()?;

        self.backend.seek(SeekFrom::Start(
            self.dynamic_header.bat_offset + 4 * bat_index as u64,
        ))?;
        self.backend.write_u32::<BigEndian>(bat_value)?;
        self.free_data_block_offset = next_offset;

        Ok(self.get_offset(offset, false)?.unwrap().unwrap())
    }

    fn rewrite_footer(&mut self) -> io::Result<()> {
        if !self.footer_encoded_valid {
            self.footer.encode(&mut self.footer_encoded);
            self.footer_encoded_valid = true;
        }

        self.backend.write_all(&self.footer_encoded)
    }
}

impl Disk for VhdDisk {
    fn disk_size(&self) -> u64 {
        self.max_disk_size as u64
    }
    fn sector_size(&self) -> u32 {
        512
    }
    fn media_type(&self) -> MediaType {
        MediaType::HDD
    }
    fn disk_format(&self) -> DiskFormat {
        DiskFormat::VHD
    }
}

impl Seek for VhdDisk {
    fn seek(&mut self, s: SeekFrom) -> io::Result<u64> {
        match s {
            SeekFrom::Start(x) => self.cursor = x,
            SeekFrom::Current(x) => self.cursor = self.cursor.wrapping_add(x as u64),
            _ => unimplemented!(),
        }

        Ok(self.cursor)
    }
}

impl Read for VhdDisk {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut left = buf.len();
        let mut total_read = 0usize;
        while left > 0 {
            let n = min(
                left,
                self.block_size as usize - (self.cursor as usize % self.block_size as usize),
            );

            if let Some(offset_in_file) = self.get_offset(self.cursor, false)? {
                if let Some(offset_in_file) = offset_in_file {
                    self.backend.seek(SeekFrom::Start(offset_in_file))?;
                    self.backend
                        .read_exact(&mut buf[total_read..total_read + n])?;
                } else {
                    zero_u8_slice(&mut buf[total_read..total_read + n]);
                }
            } else {
                break;
            }

            left -= n;
            self.cursor += n as u64;
            total_read += n;
        }

        Ok(total_read)
    }
}

impl Write for VhdDisk {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut left = buf.len();
        let mut total_written = 0usize;
        while left > 0 {
            let n = min(
                left,
                self.block_size as usize - (self.cursor as usize % self.block_size as usize),
            );

            // TODO: optimize this
            let mut is_zero = true;
            for x in buf[total_written..total_written + n].iter().copied() {
                if x != 0 {
                    is_zero = false;
                    break;
                }
            }

            if let Some(offset_in_file) = self.get_offset(self.cursor, true)? {
                if !is_zero {
                    #[allow(clippy::redundant_closure)]
                    let offset_in_file =
                        offset_in_file.map_or_else(|| self.alloc_block(self.cursor), |x| Ok(x))?;
                    self.backend.seek(SeekFrom::Start(offset_in_file))?;
                    self.backend
                        .write_all(&buf[total_written..total_written + n])?;
                } else if let Some(offset_in_file) = offset_in_file {
                    self.backend.seek(SeekFrom::Start(offset_in_file))?;
                    self.backend
                        .write_all(&buf[total_written..total_written + n])?;
                }
            } else {
                break;
            }

            left -= n;
            self.cursor += n as u64;
            total_written += n;
        }

        Ok(total_written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.backend.flush()
    }
}
