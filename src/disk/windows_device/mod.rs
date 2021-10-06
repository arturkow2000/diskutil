use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::mem::{size_of, MaybeUninit};
use std::os::windows::io::AsRawHandle;
use std::path::Path;
use std::ptr;

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::ioapiset::DeviceIoControl;
use winapi::um::winioctl::{DISK_GEOMETRY, IOCTL_DISK_GET_DRIVE_GEOMETRY};

use super::Backend;
use crate::{Error, Result};

pub struct DeviceBackend {
    disk: File,
    size: u64,
}

impl DeviceBackend {
    pub fn new(path: &Path, write: bool) -> Result<Box<Self>> {
        let disk = OpenOptions::new().read(true).write(write).open(path)?;
        let handle = disk.as_raw_handle();

        let size = unsafe {
            let mut geom: MaybeUninit<DISK_GEOMETRY> = MaybeUninit::uninit();
            let mut junk: MaybeUninit<u32> = MaybeUninit::uninit();

            // FIXME: IOCTL_DISK_GET_DRIVE_GEOMETRY returns logical sector sector (always 512 bytes) instead of physical
            if DeviceIoControl(
                handle,
                IOCTL_DISK_GET_DRIVE_GEOMETRY,
                ptr::null_mut(),
                0,
                geom.as_mut_ptr() as _,
                size_of::<DISK_GEOMETRY>().try_into().unwrap(),
                junk.as_mut_ptr(),
                ptr::null_mut(),
            ) == 0
            {
                return Err(Error::IoError(io::Error::from_raw_os_error(
                    GetLastError() as i32
                )));
            }

            let geom = geom.assume_init();

            let cylinders = *geom.Cylinders.QuadPart() as u64;
            cylinders
                * geom.TracksPerCylinder as u64
                * geom.SectorsPerTrack as u64
                * geom.BytesPerSector as u64
        };

        Ok(Box::new(Self { disk, size }))
    }
}

impl Read for DeviceBackend {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.disk.read(buf) {
            Ok(x) => Ok(x),
            Err(e) => {
                error!("read({}) {}", buf.len(), e);
                Err(e)
            }
        }
    }
}

impl Write for DeviceBackend {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.disk.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.disk.flush()
    }
}

impl Seek for DeviceBackend {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match pos {
            SeekFrom::Start(p) => debug!("abs {}", p),
            SeekFrom::End(p) => debug!("end {}", p),
            SeekFrom::Current(p) => debug!("rel {}", p),
        };
        self.disk.seek(pos)
    }
}

impl Backend for DeviceBackend {
    fn data_length(&self) -> u64 {
        self.size
    }
}
