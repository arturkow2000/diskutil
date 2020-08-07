use crate::disk::vhd::DiskType;
use crate::{u8_array_uninitialized, Error, Result};
use chrono::prelude::*;
use std::convert::{TryFrom, TryInto};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::time::SystemTime;
use std::{fmt, str};
use uuid::Uuid;

pub struct Footer {
    pub features: u32,
    pub version: u32,
    pub data_offset: u64,
    pub time_stamp: u32,
    pub creator_app: [u8; 4],
    pub creator_version: u32,
    pub creator_host_os: [u8; 4],
    pub original_size: u64,
    pub current_size: u64,
    pub disk_geometry: u32,
    pub disk_type: DiskType,
    pub uuid: Uuid,
    pub saved_state: u8,
    pub reserved: [u8; 427],
}
impl Footer {
    pub const SIZE: usize = 512;

    pub fn decode(buffer: &[u8]) -> Result<Self> {
        debug_assert_eq!(buffer.len(), Self::SIZE);

        let mut reader = Cursor::new(buffer);

        let mut temp_buffer = u8_array_uninitialized!(16);
        #[cfg(debug_assertions)]
        let mut computed_checksum = 0u32;

        let compute_checksum = |checksum: &mut u32, b: &[u8]| {
            for x in b.iter().copied() {
                *checksum = checksum.wrapping_add(x.into())
            }
        };

        macro_rules! read {
            (u8) => {{
                reader.read_exact(&mut temp_buffer[..1])?;
                compute_checksum(&mut computed_checksum, &temp_buffer[..1]);
                temp_buffer[0]
            }};
            ($type:ty) => {{
                let s = &mut temp_buffer[..::std::mem::size_of::<$type>()];
                reader.read_exact(s)?;
                compute_checksum(&mut computed_checksum, s);
                <$type>::from_be_bytes((*s).try_into().unwrap())
            }};
            ($type:ty, native) => {{
                let s = &mut temp_buffer[..::std::mem::size_of::<$type>()];
                reader.read_exact(s)?;
                compute_checksum(&mut computed_checksum, s);
                <$type>::from_ne_bytes((*s).try_into().unwrap())
            }};
            ($type:ty, nohash) => {{
                let s = &mut temp_buffer[..::std::mem::size_of::<$type>()];
                reader.read_exact(s)?;
                let z = [0u8; ::std::mem::size_of::<$type>()];
                compute_checksum(&mut computed_checksum, &z[..]);
                <$type>::from_be_bytes((*s).try_into().unwrap())
            }};
            ($size:expr) => {{
                let mut b = u8_array_uninitialized!($size);
                reader.read_exact(&mut b[..])?;
                compute_checksum(&mut computed_checksum, &b[..]);
                b
            }};
        }

        let cookie = read!(8);
        if &cookie != b"conectix" {
            return Err(Error::InvalidVhdFooter(Some("Invalid cookie".to_owned())));
        }

        let features = read!(u32);
        let version = read!(u32);
        let data_offset = read!(u64);
        let time_stamp = read!(u32);
        let creator_app = read!(4);
        let creator_version = read!(u32);
        let creator_host_os = read!(4);
        let original_size = read!(u64);
        let current_size = read!(u64);
        let disk_geometry = read!(u32);
        let disk_type = DiskType::try_from(read!(u32))?;
        let checksum = read!(u32, nohash);
        let uuid = Uuid::from_u128(read!(u128, native));
        let saved_state = read!(u8);
        let reserved = read!(427);

        debug_assert_eq!(reader.position(), Self::SIZE as u64);

        computed_checksum = !computed_checksum;

        if checksum != computed_checksum {
            return Err(Error::InvalidVhdFooter(Some(format!(
                "Checksum mismatch, computed 0x{:08X} but the checksum is 0x{:08X}",
                computed_checksum, checksum
            ))));
        }

        Ok(Self {
            features,
            version,
            data_offset,
            time_stamp,
            creator_app,
            creator_version,
            creator_host_os,
            original_size,
            current_size,
            disk_geometry,
            disk_type,
            uuid,
            saved_state,
            reserved,
        })
    }

    pub fn create(disk_type: DiskType, max_sectors: usize) -> Self {
        let total_size = max_sectors as u64 * 512;
        let (c, h, s) = Self::compute_chs(max_sectors);
        let disk_geometry = ((s as u32) << 24) | ((h as u32) << 16) | (c as u32);

        Self {
            features: 2,
            version: 0x10000,
            data_offset: match disk_type {
                DiskType::Dynamic | DiskType::Differencing => 512,
                DiskType::Fixed => 0xFFFFFFFFFFFFFFFFu64,
            },
            time_stamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_or(0, |x| (x.as_secs() - 946684800).try_into().unwrap_or(0)),
            creator_app: *b"rvd ",
            creator_version: 1u32,
            creator_host_os: *b"Wi2k",
            original_size: total_size,
            current_size: total_size,
            disk_geometry,
            disk_type,
            uuid: Uuid::new_v4(),
            saved_state: 0,
            reserved: [0; 427],
        }
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

    pub fn encode(&self, buf: &mut [u8]) {
        let mut cursor = Cursor::new(buf);
        let mut checksum: u32 = 0;

        let compute_checksum = |checksum: &mut u32, b: &[u8]| {
            for x in b.iter().copied() {
                *checksum = checksum.wrapping_add(x.into())
            }
        };

        macro_rules! write {
            ($data:expr, array) => {{
                compute_checksum(&mut checksum, $data);
                cursor.write_all($data).unwrap();
            }};
            ($data:expr) => {{
                let x = $data.to_be_bytes();
                compute_checksum(&mut checksum, &x);
                cursor.write_all(&x).unwrap();
            }};
        }

        write!(b"conectix", array);
        write!(self.features);
        write!(self.version);
        write!(self.data_offset);
        write!(self.time_stamp);
        write!(&self.creator_app, array);
        write!(self.creator_version);
        write!(&self.creator_host_os, array);
        write!(self.original_size);
        write!(self.current_size);
        write!(self.disk_geometry);
        write!(self.disk_type as u32);
        let p = cursor.position();
        write!(0u32);
        write!(self.uuid.as_u128());
        write!(&[self.saved_state], array);
        write!(&self.reserved, array);

        debug_assert_eq!(cursor.position(), Footer::SIZE as u64);

        checksum = !checksum;
        cursor.seek(SeekFrom::Start(p)).unwrap();
        write!(checksum);
    }
}

impl fmt::Display for Footer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Features              : 0x{:08X}
Version               : 0x{:08X}
Data Offset           : 0x{:016X}
Creation Date         : {}
Creator               : {}
Creator Version       : 0x{:08X}
Creator Host OS       : {}
Original Size         : 0x{:016X}
Current Size          : 0x{:016X}
Disk Type             : {:?}
UUID                  : {{{}}}",
            self.features,
            self.version,
            self.data_offset,
            Local.timestamp(self.time_stamp as i64 + 946684800, 0),
            str::from_utf8(&self.creator_app).unwrap_or("<invalid>"),
            self.creator_version,
            str::from_utf8(&self.creator_host_os).map_or_else(
                |_| format!("0x{:08X}", u32::from_be_bytes(self.creator_host_os)),
                |x| x.to_owned()
            ),
            self.original_size,
            self.current_size,
            self.disk_type,
            self.uuid
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Footer;
    use crate::disk::vhd::DiskType;
    use std::str::FromStr;
    use uuid::Uuid;

    static FOOTER: [u8; Footer::SIZE] = [
        0x63, 0x6f, 0x6e, 0x65, 0x63, 0x74, 0x69, 0x78, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x72, 0x76,
        0x64, 0x20, 0x00, 0x05, 0x00, 0x03, 0x57, 0x69, 0x32, 0x6b, 0x00, 0x00, 0x00, 0x00, 0x02,
        0xb0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xb0, 0x00, 0x00, 0x11, 0x06, 0x03, 0x5f,
        0x00, 0x00, 0x00, 0x03, 0xff, 0xff, 0xf7, 0xec, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];

    #[test]
    fn test_decode() {
        crate::tests_init();

        let footer = Footer::decode(&FOOTER).unwrap();
        assert_eq!(footer.features, 2);
        assert_eq!(footer.version, 0x10000);
        assert_eq!(footer.data_offset, 512);
        // TODO: time stamp check
        assert_eq!(&footer.creator_app, b"rvd ");
        assert_eq!(footer.creator_version, (5u32 << 16) | 3u32);
        assert_eq!(&footer.creator_host_os, b"Wi2k");
        assert_eq!(footer.original_size, 45088768);
        assert_eq!(footer.current_size, 45088768);
        assert_eq!(footer.disk_type, DiskType::Dynamic);
        assert_eq!(
            footer.uuid,
            Uuid::from_str("00000000-0000-0000-0000-000000000000").unwrap()
        );
        assert_eq!(footer.saved_state, 0);
        assert_eq!(footer.reserved, [0; 427]);
    }

    #[test]
    fn test_encode() {
        crate::tests_init();

        let footer = Footer::decode(&FOOTER).unwrap();
        let mut buffer = [0u8; Footer::SIZE];
        footer.encode(&mut buffer[..]);
        assert_eq!(buffer, FOOTER);
    }
}
