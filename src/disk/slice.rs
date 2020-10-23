use crate::disk::{Disk, DiskFormat, MediaType};
use std::cmp::min;
use std::io::{self, Read, Seek, SeekFrom, Write};

pub struct DiskSlice<'a> {
    parent: &'a mut dyn Disk,
    // first byte that belongs to this slice
    start: u64,
    // last byte that belongs to this slice
    end: u64,
    cursor: u64,
}

impl<'a> DiskSlice<'a> {
    pub fn new(parent: &'a mut dyn Disk, first_sector: u64, num_sectors: u64) -> Self {
        assert_ne!(num_sectors, 0);

        let disk_size = parent.disk_size();
        let sector_size = parent.sector_size();

        let last_sector = first_sector + num_sectors;

        assert!(last_sector * sector_size as u64 <= disk_size);

        let start = first_sector * sector_size as u64;
        let end = (last_sector * sector_size as u64) - 1;

        Self {
            parent,
            start,
            end,
            cursor: 0,
        }
    }

    pub fn parent(&self) -> &dyn Disk {
        self.parent
    }
    pub fn parent_mut(&mut self) -> &mut dyn Disk {
        self.parent
    }
}

impl<'a> Read for DiskSlice<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.parent
            .seek(SeekFrom::Start(self.start + self.cursor))?;

        let total_available = (self.end + 1).saturating_sub(self.cursor + self.start);

        if total_available == 0 {
            return Ok(0);
        }

        let to_read = min(total_available, buf.len() as u64);
        let r = self.parent.read(&mut buf[..to_read as usize])?;
        self.cursor += r as u64;
        Ok(r)
    }
}

impl<'a> Seek for DiskSlice<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(x) => self.cursor = x,
            SeekFrom::End(x) => self.cursor = (self.end + 1) - x as u64,
            SeekFrom::Current(x) => self.cursor = self.cursor.wrapping_add(x as u64),
        }

        Ok(self.cursor)
    }
}

impl<'a> Write for DiskSlice<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.parent
            .seek(SeekFrom::Start(self.start + self.cursor))?;

        let total_available = (self.end + 1).saturating_sub(self.cursor + self.start);

        if total_available == 0 {
            return Ok(0);
        }

        let to_write = min(total_available, buf.len() as u64);
        let w = self.parent.write(&buf[..to_write as usize])?;
        self.cursor += w as u64;
        Ok(w)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.parent.flush()
    }
}

impl<'a> Disk for DiskSlice<'a> {
    fn disk_size(&self) -> u64 {
        (self.end - self.start) + 1
    }
    fn sector_size(&self) -> u32 {
        self.parent.sector_size()
    }
    fn media_type(&self) -> MediaType {
        self.parent.media_type()
    }
    fn disk_format(&self) -> DiskFormat {
        self.parent.disk_format()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disk::ram::RamDisk;
    use crate::u8_array_uninitialized;

    #[test]
    fn test_disk_slice_creation() {
        let mut disk = RamDisk::new_uninitialized(512, 3);
        assert_eq!(DiskSlice::new(&mut disk, 0, 1).disk_size(), 512);
        assert_eq!(DiskSlice::new(&mut disk, 0, 2).disk_size(), 1024);
        assert_eq!(DiskSlice::new(&mut disk, 0, 3).disk_size(), 1536);
        assert_eq!(DiskSlice::new(&mut disk, 1, 2).disk_size(), 1024);
    }

    #[test]
    #[should_panic]
    fn test_disk_slice_creation_out_of_bounds() {
        let mut disk = RamDisk::new_uninitialized(512, 3);
        let _ = DiskSlice::new(&mut disk, 0, 4);
    }

    #[test]
    #[should_panic]
    fn test_disk_slice_creation_zero_size() {
        let mut disk = RamDisk::new_uninitialized(512, 1);
        let _ = DiskSlice::new(&mut disk, 0, 0);
    }

    fn create_test_disk() -> RamDisk {
        let mut disk = RamDisk::new_zeroed(512, 3);
        disk.write_all(b"1245P21").unwrap();

        disk.seek(SeekFrom::Start(512)).unwrap();
        disk.write_all(b"0021X11R97").unwrap();
        disk.seek(SeekFrom::Start(1020)).unwrap();
        disk.write_all(b"A4N1").unwrap();

        disk.write_all(b"@@@@@@@@@@@@@@@@@@@@@@@@@").unwrap();

        disk
    }

    macro_rules! read {
        ($r:expr, $s:expr) => {{
            let mut b = u8_array_uninitialized!($s);
            $r.read_exact(&mut b[..]).unwrap();
            b
        }};
    }

    macro_rules! test_read_partial {
        ($r:expr, $s:expr, $expected:expr) => {{
            let mut b = u8_array_uninitialized!($s);
            let n = $r.read(&mut b[..]).unwrap();
            assert_eq!(&b[..n], $expected);
        }};
    }

    #[test]
    fn test_disk_slice_seek() {
        let mut disk = RamDisk::new_uninitialized(512, 3);
        let mut slice = DiskSlice::new(&mut disk, 1, 1);

        macro_rules! test_seek {
            ($pos:expr, $expected:expr) => {{
                slice.seek($pos).unwrap();
                let mut b: [u8; 0] = [];
                // do read so DiskSlice seeks on parent
                let _ = slice.read(&mut b[..]);

                assert_eq!(
                    slice.parent_mut().seek(SeekFrom::Current(0)).unwrap(),
                    $expected
                );
            }};
        }

        test_seek!(SeekFrom::Start(0), 512);
        test_seek!(SeekFrom::Current(4), 516);
        test_seek!(SeekFrom::Current(-3), 513);
        test_seek!(SeekFrom::Start(20000), 20512);
        test_seek!(SeekFrom::End(0), 1536);
        test_seek!(SeekFrom::Current(2), 1538);
    }

    #[test]
    fn test_disk_slice_read() {
        let mut disk = create_test_disk();

        let mut slice = DiskSlice::new(&mut disk, 0, 1);

        assert_eq!(&read!(slice, 7), b"1245P21");

        slice.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(&read!(slice, 7), b"1245P21");

        slice.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(&read!(slice, 4), b"1245");
        assert_eq!(&read!(slice, 3), b"P21");

        let mut slice = DiskSlice::new(&mut disk, 1, 1);
        assert_eq!(&read!(slice, 3), b"002");

        slice.seek(SeekFrom::Start(508)).unwrap();
        //assert_eq!(&read!(slice, 4), b"A4N1");
        test_read_partial!(slice, 4, b"A4N1");
        slice.seek(SeekFrom::Current(-4)).unwrap();
        test_read_partial!(slice, 20, b"A4N1");
        test_read_partial!(slice, 20, b"");
    }

    #[test]
    fn test_disk_slice_write() {
        let mut disk = create_test_disk();
        let mut slice = DiskSlice::new(&mut disk, 1, 1);
        slice.write_all(b"test").unwrap();
        slice.seek(SeekFrom::Start(0)).unwrap();
        assert_eq!(&read!(slice, 4), b"test");

        assert_eq!(slice.seek(SeekFrom::Start(510)).unwrap(), 510);
        assert_eq!(slice.write(b"443434343434").unwrap(), 2);
    }
}
