use crate::io::Write;

pub enum SeekFrom {
    Start(usize),
    End(isize),
    Current(isize),
}

pub struct TrackedSlice<'a> {
    buffer: &'a mut [u8],
    bytes_written: usize,
}

impl<'a> TrackedSlice<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            bytes_written: 0,
        }
    }

    pub fn new_in(buffer: &'a mut [u8], bytes_written: usize) -> Option<Self> {
        if bytes_written > buffer.len() {
            None
        } else {
            Some(Self {
                buffer,
                bytes_written,
            })
        }
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.bytes_written
    }

    #[inline]
    pub fn full(&self) -> bool {
        self.bytes_written == self.buffer.len()
    }

    #[inline]
    pub fn bytes_written(&self) -> usize {
        self.bytes_written
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    #[inline]
    pub fn seek(&mut self, seek: SeekFrom) -> bool {
        match seek {
            SeekFrom::Start(offset) => {
                if offset > self.buffer.len() {
                    false
                } else {
                    self.bytes_written = offset;
                    true
                }
            }
            SeekFrom::End(offset) => {
                let new_offset = ((self.buffer.len() as isize) + offset) as usize;
                if new_offset > self.buffer.len() {
                    false
                } else {
                    self.bytes_written = new_offset;
                    true
                }
            }
            SeekFrom::Current(offset) => {
                let new_offset = ((self.bytes_written as isize) + offset) as usize;

                if new_offset > self.buffer.len() {
                    false
                } else {
                    self.bytes_written = new_offset;
                    true
                }
            }
        }
    }
}

impl<'a> Write for TrackedSlice<'a> {
    fn write(&mut self, buf: &[u8]) -> anyhow::Result<usize> {
        let write_size = core::cmp::min(buf.len(), self.remaining());

        unsafe {
            let dst = self.buffer.as_mut_ptr().add(self.bytes_written);
            core::ptr::copy(buf.as_ptr(), dst, write_size);
        }
        self.bytes_written += write_size;
        Ok(write_size)
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}
