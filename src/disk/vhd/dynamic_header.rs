use crate::{disk::vhd::DiskType, Error, Result};
use std::mem::{size_of, transmute, zeroed};
use std::{fmt, slice, str};

#[repr(C)]
pub struct DynamicHeader {
    pub __cookie: [u8; 8],
    pub __data_offset: u64,
    pub __bat_offset: u64,
    pub __header_version: u32,
    pub __max_table_entries: u32,
    pub __block_size: u32,
    pub __checksum: u32,
    pub __parent_unique_id: [u8; 16],
    pub __parent_timestamp: u32,
    pub __reserved: u32,
    pub __parent_unicode_name: [u8; 512],
    pub __parent_locator_entry_1: [u8; 24],
    pub __parent_locator_entry_2: [u8; 24],
    pub __parent_locator_entry_3: [u8; 24],
    pub __parent_locator_entry_4: [u8; 24],
    pub __parent_locator_entry_5: [u8; 24],
    pub __parent_locator_entry_6: [u8; 24],
    pub __parent_locator_entry_7: [u8; 24],
    pub __parent_locator_entry_8: [u8; 24],
    pub __reserved2: [u8; 256],
}

impl Default for DynamicHeader {
    fn default() -> Self {
        unsafe { zeroed() }
    }
}

#[inline(always)]
fn __test_size() {
    unsafe { transmute::<[u8; 1024], DynamicHeader>([0u8; 1024]) };
}

impl fmt::Display for DynamicHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header_version = self.header_version();

        // TODO: print other fields
        write!(
            f,
            "Cookie                : {}
Data Offset           : 0x{:08X}
BAT Offset            : {}
Header Version        : {}.{}
Max Table Entries     : {}
Block Size            : 0x{:08X}",
            str::from_utf8(&self.__cookie).unwrap_or("<invalid>"),
            self.data_offset(),
            self.bat_offset(),
            header_version.0,
            header_version.1,
            self.max_table_entries(),
            self.block_size()
        )
    }
}

impl DynamicHeader {
    #[inline]
    pub fn data_offset(&self) -> u64 {
        u64::from_be(self.__data_offset)
    }

    #[inline]
    pub fn bat_offset(&self) -> u64 {
        u64::from_be(self.__bat_offset)
    }

    #[inline]
    pub fn header_version(&self) -> (u16, u16) {
        let v = u32::from_be(self.__header_version);
        ((v >> 16) as u16, (v & 0xFFFF) as u16)
    }

    #[inline]
    pub fn max_table_entries(&self) -> u32 {
        u32::from_be(self.__max_table_entries)
    }

    #[inline]
    pub fn block_size(&self) -> u32 {
        u32::from_be(self.__block_size)
    }

    pub fn compute_checksum(&self) -> u32 {
        let a = unsafe { transmute::<&Self, &[u8; size_of::<Self>()]>(self) };
        let mut checksum: u32 = 0;

        for (i, mut b) in a.iter().copied().enumerate() {
            if i & !0b11 == 36 {
                b = 0;
            }

            checksum = checksum.wrapping_add(b.into());
        }

        !checksum
    }

    pub fn verify(&self, disk_type: DiskType, file_size: usize) -> Result<()> {
        let computed_checksum = self.compute_checksum();
        let checksum = u32::from_be(self.__checksum);
        if computed_checksum != checksum {
            return Err(Error::InvalidVhdDynamicHeader(Some(format!(
                "Invalid checksum, checksum=0x{:08X} computed_checksum=0x{:08X}",
                checksum, computed_checksum
            ))));
        }

        if self.__cookie != [b'c', b'x', b's', b'p', b'a', b'r', b's', b'e'] {
            return Err(Error::InvalidVhdDynamicHeader(Some(
                "Invalid cookie.".to_owned(),
            )));
        }

        let version = self.header_version();
        if version.0 != 1 {
            return Err(Error::InvalidVhdDynamicHeader(Some(format!(
                "Unsupported version, expected 1.x got {}.{}.",
                version.0, version.1
            ))));
        }

        if self.data_offset() != 0xFFFFFFFFFFFFFFFF {
            return Err(Error::InvalidVhdDynamicHeader(Some(
                "Invalid data_offset".to_owned(),
            )));
        }

        let bat_offset = self.bat_offset() as usize;
        if bat_offset > file_size {
            return Err(Error::InvalidVhdDynamicHeader(Some(format!(
                "BAT offset points past end of file ({} > {})",
                bat_offset, file_size
            ))));
        }

        if disk_type == DiskType::Differencing {
            return Err(Error::InvalidVhdDynamicHeader(Some(
                "Differencing disks not supported yet.".to_owned(),
            )));
        }

        assert_eq!(disk_type, DiskType::Dynamic);

        Ok(())
    }

    pub fn create_dynamic(bat_size: usize, block_size: usize) -> Self {
        let mut this = Self::default();

        this.__cookie.copy_from_slice(b"cxsparse");
        this.__data_offset = 0xFFFFFFFFFFFFFFFFu64;
        this.__bat_offset = u64::to_be(1536);
        this.__header_version = u32::to_be(0x00010000);
        this.__block_size = u32::to_be(block_size as u32);
        this.__max_table_entries = u32::to_be(bat_size as u32);

        this.__checksum = u32::to_be(this.compute_checksum());

        this
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }
    }
}
