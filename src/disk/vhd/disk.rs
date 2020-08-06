use crate::disk::vhd::{dynamic_header::DynamicHeader, footer::Footer, DiskType as VhdDiskType};
use crate::disk::{Disk, DiskType, Info, MediaType};
#[macro_use]
use crate::{is_power_of_2, round_up, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::cmp::min;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::mem::{size_of, transmute, MaybeUninit};
use std::{ptr, slice};

const SECTOR_SIZE: u32 = 512;

pub struct VhdDisk<B>
where
    B: Read + Seek + Write,
{
    backend: B,
    footer: Footer,
    dynamic_header: DynamicHeader,
    bat: Vec<u32>,

    block_size: u32,
    bitmap_size: u32,
    max_disk_size: usize,

    cursor: u64,
    free_data_block_offset: u64,
}

// TODO: could use write_all_vectored to increase performance https://github.com/rust-lang/rust/issues/70436
impl<B> VhdDisk<B>
where
    B: Read + Seek + Write,
{
    pub fn open(backend: B) -> Result<Self> {
        let mut this = Self {
            backend,
            footer: unsafe { MaybeUninit::uninit().assume_init() },
            dynamic_header: unsafe { MaybeUninit::uninit().assume_init() },
            bat: Vec::new(),
            block_size: 0,
            bitmap_size: 0,
            max_disk_size: 0,
            cursor: 0,
            free_data_block_offset: !0,
        };

        let file_size = this
            .backend
            .seek(SeekFrom::End(-(size_of::<Footer>() as i64)))? as usize
            + size_of::<Footer>();
        this.backend.read_exact(unsafe {
            &mut transmute::<&mut Footer, &mut [u8; size_of::<Footer>()]>(&mut this.footer)[..]
        })?;

        debug!("{}", this.footer);

        this.footer.verify(file_size)?;

        this.backend
            .seek(SeekFrom::Start(this.footer.data_offset()))?;
        this.backend.read_exact(unsafe {
            &mut transmute::<&mut DynamicHeader, &mut [u8; size_of::<Footer>()]>(
                &mut this.dynamic_header,
            )[..]
        })?;

        debug!("{}", this.dynamic_header);

        //this.dynamic_header
        //    .verify(this.footer.disk_type(), file_size)?;

        this.backend
            .seek(SeekFrom::Start(this.dynamic_header.bat_offset()))?;

        let bat_size = this.dynamic_header.max_table_entries() as usize;
        this.bat.reserve(bat_size);
        unsafe { this.bat.set_len(bat_size) };
        this.backend
            .read_u32_into::<BigEndian>(this.bat.as_mut_slice())?;

        this.block_size = this.dynamic_header.block_size();
        this.bitmap_size = ((this.block_size / (8 * 512)) + 511) & !511;
        this.max_disk_size = this.dynamic_header.max_table_entries() as usize
            * this.dynamic_header.block_size() as usize;

        this.free_data_block_offset = round_up!(
            this.dynamic_header.bat_offset() + this.bat.len() as u64 * 4,
            SECTOR_SIZE as u64
        );

        Ok(this)
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

    pub fn create_dynamic(backend: B, max_sectors: usize) -> io::Result<Self> {
        Self::create_dynamic_ex(backend, max_sectors, 1024 * 1024 * 2)
    }
    pub fn create_dynamic_ex(
        mut backend: B,
        max_sectors: usize,
        block_size: usize,
    ) -> io::Result<Self> {
        // TODO: verify max_sectors and block_size

        let footer = Footer::create(VhdDiskType::Dynamic, max_sectors);
        backend.write_all(footer.as_bytes())?;

        let mut bat_size = max_sectors / (block_size / SECTOR_SIZE as usize);
        if max_sectors % (block_size / SECTOR_SIZE as usize) != 0 {
            bat_size += 1;
        }

        let dynamic_header = DynamicHeader::create_dynamic(bat_size, block_size);
        backend.write_all(dynamic_header.as_bytes())?;

        let mut bat: Vec<u32> = Vec::new();
        bat.resize_with(bat_size, || 0xFFFFFFFF);

        unsafe {
            let bat_u8s: &[u8] = slice::from_raw_parts(bat.as_ptr() as *const u8, bat.len() * 4);
            backend.write_all(bat_u8s)?;

            if bat_u8s.len() % SECTOR_SIZE as usize != 0 {
                let padding_len = (bat_u8s.len() + 511 & !511) - bat_u8s.len();
                let mut padding: Vec<u8> = Vec::new();
                padding.resize_with(padding_len, || 0xff);
                backend.write_all(padding.as_slice())?;
            }
        }

        backend.write_all(footer.as_bytes())?;

        let free_data_block_offset = round_up!(
            dynamic_header.bat_offset() + bat.len() as u64 * 4,
            SECTOR_SIZE as u64
        );

        Ok(Self {
            backend,
            footer,
            dynamic_header,
            bat,
            block_size: block_size as u32,
            bitmap_size: ((block_size as u32 / (8 * 512)) + 511) & !511,
            max_disk_size: max_sectors * 512,
            cursor: 0,
            free_data_block_offset,
        })
    }

    pub fn alloc_block(&mut self, offset: u64) -> io::Result<u64> {
        let bat_index = offset as usize / self.block_size as usize;
        assert_eq!(self.bat[bat_index], 0xFFFFFFFF);

        let bat_value = (self.free_data_block_offset / SECTOR_SIZE as u64) as u32;
        self.bat[bat_index] = bat_value;

        let mut bitmap: Vec<u8> = Vec::new();
        bitmap.resize_with(self.bitmap_size as usize, || 0xff);
        self.backend
            .seek(SeekFrom::Start(self.free_data_block_offset))?;
        self.backend.write_all(bitmap.as_slice())?;

        let next_offset =
            self.free_data_block_offset + self.block_size as u64 + self.bitmap_size as u64;
        self.backend.seek(SeekFrom::Start(next_offset))?;
        self.backend.write_all(self.footer.as_bytes())?;

        self.backend.seek(SeekFrom::Start(
            self.dynamic_header.bat_offset() + 4 * bat_index as u64,
        ))?;
        self.backend.write_u32::<BigEndian>(bat_value)?;
        self.free_data_block_offset = next_offset;

        Ok(self.get_offset(offset, false)?.unwrap().unwrap())
    }
}

impl<B> Info for VhdDisk<B>
where
    B: Read + Seek + Write,
{
    fn disk_type(&self) -> DiskType {
        DiskType::VHD
    }
    fn max_disk_size(&self) -> usize {
        self.max_disk_size
    }
    fn disk_size(&self) -> usize {
        todo!()
    }
    fn block_size(&self) -> usize {
        512
    }
    fn media_type(&self) -> MediaType {
        MediaType::HDD
    }
}

impl<B> Seek for VhdDisk<B>
where
    B: Read + Seek + Write,
{
    fn seek(&mut self, s: SeekFrom) -> io::Result<u64> {
        match s {
            SeekFrom::Start(x) => self.cursor = x,
            SeekFrom::Current(x) => self.cursor = self.cursor.wrapping_add(x as u64),
            _ => unimplemented!(),
        }

        Ok(self.cursor)
    }
}

impl<B> Read for VhdDisk<B>
where
    B: Read + Seek + Write,
{
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
                    unsafe {
                        let s = &mut buf[total_read..total_read + n];
                        // FIXME: see https://github.com/rust-lang/rfcs/issues/2067
                        ptr::write_bytes(s.as_mut_ptr(), 0, n)
                    }
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

impl<B> Write for VhdDisk<B>
where
    B: Read + Seek + Write,
{
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
                    let offset_in_file =
                        offset_in_file.map_or_else(|| self.alloc_block(self.cursor), |x| Ok(x))?;
                    self.backend.seek(SeekFrom::Start(offset_in_file))?;
                    self.backend
                        .write_all(&buf[total_written..total_written + n])?;
                } else {
                    if let Some(offset_in_file) = offset_in_file {
                        self.backend.seek(SeekFrom::Start(offset_in_file))?;
                        self.backend
                            .write_all(&buf[total_written..total_written + n])?;
                    }
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

impl<B> Disk for VhdDisk<B> where B: Read + Seek + Write {}
