use crate::{disk::vhd::DiskType, Error, Result};
use chrono::prelude::*;
use std::mem::{size_of, transmute, zeroed};
use std::{fmt, slice, str};

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum CreatorHostOs {
    Unknown(u32),
    Windows,
    macOS,
}

impl From<u32> for CreatorHostOs {
    fn from(x: u32) -> Self {
        match x {
            0x5769326B => Self::Windows,
            0x4D616320 => Self::macOS,
            x => Self::Unknown(x),
        }
    }
}

#[repr(C)]
pub struct Footer {
    pub __cookie: [u8; 8],
    // bit 0 => temporary
    // bit 1 => reserved must be 1
    pub __features: u32,
    pub __version: u32,
    pub __data_offset: u64,
    pub __time_stamp: u32,
    pub __creator_app: [u8; 4],
    pub __creator_version: u32,
    pub __creator_host_os: u32,
    pub __original_size: u64,
    pub __current_size: u64,
    pub __disk_geometry: u32,
    pub __disk_type: u32,
    pub __checksum: u32,
    pub __uuid: [u8; 16],
    pub __saved_state: u8,
    pub __reserved: [u8; 427],
}

#[inline(always)]
fn __test_size() {
    unsafe { transmute::<[u8; 512], Footer>([0u8; 512]) };
}

impl fmt::Display for Footer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chs_to_string = |(c, h, s)| -> String { format!("{} {} {}", c, h, s) };
        write!(
            f,
            "Cookie           : {}
Features         : 0x{:08X}
Version          : {}.{}
Data Offset      : 0x{:08X}
Creation Date    : {}
Creator          : {}
Creator Version  : {}.{}
Creator Host OS  : {:?}
Original Size    : 0x{:08X}
Current Size     : 0x{:08X}
CHS              : {}
Disk Type        : {:?}",
            str::from_utf8(&self.__cookie).unwrap_or("<invalid>"),
            self.features(),
            self.version().0,
            self.version().1,
            self.data_offset(),
            self.time_stamp_to_date(),
            str::from_utf8(&self.__creator_app).unwrap_or("<invalid>"),
            self.creator_version().0,
            self.creator_version().1,
            self.creator_host_os(),
            self.original_size(),
            self.current_size(),
            chs_to_string(self.chs()),
            self.disk_type()
        )
    }
}

impl Default for Footer {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[allow(clippy::transmute_ptr_to_ptr)]
impl Footer {
    #[inline]
    pub fn features(&self) -> u32 {
        u32::from_be(self.__features)
    }
    #[inline]
    pub fn version(&self) -> (u16, u16) {
        let v = u32::from_be(self.__version);
        ((v >> 16) as u16, (v & 0xFFFF) as u16)
    }

    #[inline]
    pub fn data_offset(&self) -> u64 {
        u64::from_be(self.__data_offset)
    }

    #[inline]
    pub fn time_stamp(&self) -> u32 {
        u32::from_be(self.__time_stamp)
    }

    #[inline]
    pub fn time_stamp_to_date(&self) -> DateTime<Local> {
        Local.timestamp(self.time_stamp() as i64 + 946684800, 0)
    }

    #[inline]
    pub fn creator_version(&self) -> (u16, u16) {
        let v = u32::from_be(self.__creator_version);
        ((v >> 16) as u16, (v & 0xFFFF) as u16)
    }

    #[inline]
    pub fn creator_host_os(&self) -> CreatorHostOs {
        CreatorHostOs::from(u32::from_be(self.__creator_host_os))
    }

    #[inline]
    pub fn original_size(&self) -> u64 {
        u64::from_be(self.__original_size)
    }

    #[inline]
    pub fn current_size(&self) -> u64 {
        u64::from_be(self.__current_size)
    }

    #[inline]
    pub fn chs(&self) -> (u16, u8, u8) {
        let c = u16::from_be(((self.__disk_geometry & 0xFFFF0000) >> 16) as u16);
        let h = ((self.__disk_geometry & 0xFF00) >> 8) as u8;
        let s = (self.__disk_geometry & 0xFF) as u8;

        (c, h, s)
    }

    #[inline]
    pub fn disk_type(&self) -> DiskType {
        DiskType::from(u32::from_be(self.__disk_type))
    }

    pub fn compute_checksum(&self) -> u32 {
        let a = unsafe { transmute::<&Self, &[u8; size_of::<Self>()]>(self) };
        let mut checksum: u32 = 0;

        for (i, mut b) in a.iter().copied().enumerate() {
            if i & !0b11 == 64 {
                b = 0;
            }

            checksum = checksum.wrapping_add(b.into());
        }

        !checksum
    }

    pub fn verify(&self, file_size: usize) -> Result<()> {
        let computed_checksum = self.compute_checksum();
        let checksum = u32::from_be(self.__checksum);
        if computed_checksum != checksum {
            return Err(Error::InvalidVhdFooter(Some(format!(
                "Invalid checksum, checksum=0x{:08X} computed_checksum=0x{:08X}",
                checksum, computed_checksum
            ))));
        }

        if self.features() & 2 == 0 {
            return Err(Error::InvalidVhdFooter(None));
        }

        let version = self.version();
        if version.0 != 1 {
            return Err(Error::InvalidVhdFooter(Some(format!(
                "Unsupported version, expected 1.x got {}.{}.",
                version.0, version.1
            ))));
        }

        let disk_type = self.disk_type();
        match disk_type {
            DiskType::Unknown(x) => {
                return Err(Error::InvalidVhdFooter(Some(format!(
                    "Unknown disk type {:#X}.",
                    x
                ))))
            }
            DiskType::None => {
                return Err(Error::InvalidVhdFooter(Some(
                    "Disk type \"none\" not supported.".to_owned(),
                )))
            }
            _ => (),
        };

        if disk_type == DiskType::Fixed {
            if self.__data_offset != 0xFFFFFFFF {
                return Err(Error::InvalidVhdFooter(Some(
                    "Invalid data_offset, for fixed disks should be 0xFFFFFFFF.".to_owned(),
                )));
            }
        } else {
            let offset = self.data_offset() as usize;
            if offset > file_size {
                return Err(Error::InvalidVhdFooter(Some(format!(
                    "Data offset points past end of file ({} > {})",
                    offset, file_size
                ))));
            }
        }

        Ok(())
    }

    fn compute_chs(mut total_sectors: usize) -> (u16, u8, u8) {
        let mut sectors_per_track;
        let mut heads;
        let mut cylinder_times_head;

        if total_sectors > 65535 * 16 * 255 {
            total_sectors = 65535 * 16 * 255;
        }

        if total_sectors >= 65535 * 16 * 63 {
            sectors_per_track = 255;
            heads = 16;
            cylinder_times_head = total_sectors / sectors_per_track;
        } else {
            sectors_per_track = 17;
            cylinder_times_head = total_sectors / sectors_per_track;

            heads = (cylinder_times_head + 1023) / 1024;

            if heads < 4 {
                heads = 4;
            }
            if cylinder_times_head >= (heads * 1024) || heads > 16 {
                sectors_per_track = 31;
                heads = 16;
                cylinder_times_head = total_sectors / sectors_per_track;
            }
            if cylinder_times_head >= (heads * 1024) {
                sectors_per_track = 63;
                heads = 16;
                cylinder_times_head = total_sectors / sectors_per_track;
            }
        }
        let cylinders = cylinder_times_head / heads;

        (cylinders as u16, heads as u8, sectors_per_track as u8)
    }

    pub fn create(disk_type: DiskType, max_sectors: usize) -> Self {
        let total_size = max_sectors as u64 * 512;

        let mut this = Self::default();
        this.__cookie
            .copy_from_slice(&[b'c', b'o', b'n', b'e', b'c', b't', b'i', b'x']);
        this.__creator_app
            .copy_from_slice(&[b'r', b'v', b'd', b' ']);
        this.__creator_host_os = u32::to_be(0x5769326B);
        this.__features = u32::to_be(2);
        this.__version = u32::to_be(0x00010000);
        if disk_type == DiskType::Dynamic || disk_type == DiskType::Differencing {
            this.__data_offset = u64::to_be(512);
        } else {
            this.__data_offset = 0xFFFFFFFFFFFFFFFFu64;
        }
        this.__creator_version = u32::to_be((5u32 << 16) | (3u32));
        this.__original_size = u64::to_be(total_size);
        this.__current_size = u64::to_be(total_size);
        this.__disk_type = u32::to_be(disk_type.into());

        let (c, h, s) = Self::compute_chs(max_sectors);
        this.__disk_geometry = ((u16::to_be(c) as u32) << 16) | (h as u32) << 8 | s as u32;

        // TODO: timestamp and uuid

        this.__checksum = u32::to_be(this.compute_checksum());

        this
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }
    }
}
