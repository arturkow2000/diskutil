use std::cmp::min;
use std::convert::TryInto;
use std::io::{self, Read, Seek, SeekFrom, Write};

use super::{Disk, DiskFormat, MediaType};
use crate::Result;

// Dummy buffer implementation, currently no caching occurs and code
// is unoptimized
// this exists to allow partial sector read/writes when working with
// physical devices
pub struct Buffer<T> {
    inner: T,
    position: u64,
    block_size: u32,
    temp: Vec<u8>,
}

impl<T> Buffer<T> {
    fn num_blocks(&self, buf: &[u8]) -> usize {
        let mut t = buf.len() / self.block_size as usize;
        if buf.len() % self.block_size as usize != 0 {
            t += 1;
        }
        t
    }
}

impl<T> Buffer<T>
where
    T: Disk,
{
    pub fn new(mut disk: T) -> Result<Self> {
        let block_size = disk.sector_size();
        let position = disk.seek(SeekFrom::Current(0))?;

        Ok(Self {
            inner: disk,
            position,
            block_size,
            temp: vec![0; block_size as usize],
        })
    }
}

impl<T> Buffer<T>
where
    T: Read,
{
    /// Read block into self.temp
    /// stream position must be aligned on block boundary
    fn read_block(&mut self) -> io::Result<bool> {
        let n = self.inner.read(self.temp.as_mut())?;
        if n == 0 {
            Ok(false)
        } else if n == self.block_size as usize {
            Ok(true)
        } else {
            // This should not happen, since disk size is always multiple of block size
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF in middle of block",
            ))
        }
    }

    /// Read block into buf
    /// buf length must equal block size
    /// stream position must be aligned on block boundary
    fn read_block_into(&mut self, buf: &mut [u8]) -> io::Result<bool> {
        debug_assert_eq!(buf.len(), self.block_size as usize);

        let n = self.inner.read(buf)?;
        if n == 0 {
            Ok(false)
        } else if n == self.block_size as usize {
            Ok(true)
        } else {
            // This should not happen, disk size is always multiple of block size
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF in middle of block (read)",
            ))
        }
    }
}

impl<T> Buffer<T>
where
    T: Write,
{
    /// Write contents of self.temp into block
    /// stream position must be aligned on block boundary
    fn write_block(&mut self) -> io::Result<bool> {
        let n = self.inner.write(self.temp.as_ref())?;
        if n == 0 {
            Ok(false)
        } else if n == self.block_size as usize {
            Ok(true)
        } else {
            // This should not happen, since disk size is always multiple of block size
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF in middle of block (write)",
            ))
        }
    }

    /// Write contents of buf into block
    /// buf length must equal block size
    /// stream position must be aligned on block boundary
    fn write_block_from(&mut self, buf: &[u8]) -> io::Result<bool> {
        debug_assert_eq!(buf.len(), self.block_size as usize);

        let n = self.inner.write(buf)?;
        if n == 0 {
            Ok(false)
        } else if n == self.block_size as usize {
            Ok(true)
        } else {
            // This should not happen, since disk size is always multiple of block size
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF in middle of block (write)",
            ))
        }
    }
}

impl<T> Disk for Buffer<T>
where
    T: Disk,
{
    fn disk_size(&self) -> u64 {
        self.inner.disk_size()
    }

    fn sector_size(&self) -> u32 {
        self.inner.sector_size()
    }

    fn media_type(&self) -> MediaType {
        self.inner.media_type()
    }

    fn disk_format(&self) -> DiskFormat {
        self.inner.disk_format()
    }
}

impl<T> Read for Buffer<T>
where
    T: Read + Seek,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.position % self.block_size as u64 == 0 && buf.len() % self.block_size as usize == 0
        {
            self.inner.read(buf)
        } else {
            // offset from block start
            let offset: u32 = (self.position % self.block_size as u64).try_into().unwrap();
            let mut blocks_left = self.num_blocks(buf);
            let mut bytes_read = 0usize;
            let mut left = buf.len();

            // align on block boundary prior to calling read_block()
            self.inner
                .seek(SeekFrom::Start(self.position - offset as u64))?;

            while blocks_left > 0 {
                let n = min(self.block_size as usize - offset as usize, left);

                let do_direct_read = n == self.block_size as usize && offset == 0;

                let eof = !if do_direct_read {
                    self.read_block_into(&mut buf[bytes_read..bytes_read + n])?
                } else {
                    self.read_block()?
                };

                if eof {
                    // EOF
                    break;
                }

                if !do_direct_read {
                    buf[bytes_read..bytes_read + n]
                        .copy_from_slice(&self.temp[offset as usize..offset as usize + n]);
                }

                left -= n;
                blocks_left -= 1;
                bytes_read += n;
                self.position += n as u64;
            }

            Ok(bytes_read)
        }
    }
}

impl<T> Write for Buffer<T>
where
    T: Read + Write + Seek,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.position % self.block_size as u64 == 0 && buf.len() % self.block_size as usize == 0
        {
            self.inner.write(buf)
        } else {
            let offset: u32 = (self.position % self.block_size as u64).try_into().unwrap();
            let mut blocks_left = self.num_blocks(buf);
            let mut bytes_written = 0usize;
            let mut left = buf.len();

            // align on block boundary prior to calling read_block() and write_block()
            self.inner
                .seek(SeekFrom::Start(self.position - offset as u64))?;

            while blocks_left > 0 {
                let n = min(self.block_size as usize - offset as usize, left);

                let do_direct_write = n == self.block_size as usize && offset == 0;

                if do_direct_write {
                    let eof = !self.write_block_from(&buf[bytes_written..bytes_written + n])?;
                    if eof {
                        break;
                    }
                } else {
                    let eof = !self.read_block()?;
                    if eof {
                        break;
                    }
                    // read_block() advances stream position, need to restore
                    // previous position to write same sector we just read
                    self.seek(SeekFrom::Current(-(self.block_size as i64)))?;

                    self.temp[offset as usize..offset as usize + n]
                        .copy_from_slice(&buf[bytes_written..bytes_written + n]);
                    if !self.write_block()? {
                        break;
                    }
                }

                left -= n;
                blocks_left -= 1;
                bytes_written += n;
                self.position += n as u64;
            }

            Ok(bytes_written)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T> Seek for Buffer<T>
where
    T: Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.position = self.inner.seek(pos)?;
        Ok(self.position)
    }
}
