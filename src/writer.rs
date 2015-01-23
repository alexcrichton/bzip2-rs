//! Writer-based compression/decompression streams

use std::io::{IoResult, IoError};

use ffi;
use raw::{Stream, Action};

/// A compression stream which will have uncompressed data written to it and
/// will write compressed data to an output stream.
pub struct BzCompressor<W> {
    stream: Stream,
    w: Option<W>,
    buf: Vec<u8>,
}

/// A compression stream which will have compressed data written to it and
/// will write uncompressed data to an output stream.
pub struct BzDecompressor<W> {
    stream: Stream,
    w: Option<W>,
    buf: Vec<u8>,
    done: bool,
}

impl<W: Writer> BzCompressor<W> {
    /// Create a new compression stream which will compress at the given level
    /// to write compress output to the give output stream.
    pub fn new(w: W, level: ::CompressionLevel) -> BzCompressor<W> {
        BzCompressor {
            stream: Stream::new_compress(level, 30),
            w: Some(w),
            buf: Vec::with_capacity(128 * 1024),
        }
    }

    fn do_write(&mut self, mut data: &[u8], action: Action) -> IoResult<()> {
        while data.len() > 0 || action != Action::Run {
            let total_in = self.stream.total_in();
            let rc = self.stream.compress_vec(data, &mut self.buf, action);
            data = &data[(self.stream.total_in() - total_in) as usize..];

            match rc {
                ffi::BZ_STREAM_END => break,
                n if n >= 0 => {}
                ffi::BZ_OUTBUFF_FULL => {
                    try!(self.w.as_mut().unwrap().write(self.buf.as_slice()));
                    self.buf.truncate(0);
                }
                n => panic!("unexpected return: {}", n),
            }
        }

        if action == Action::Finish && self.buf.len() > 0 {
            try!(self.w.as_mut().unwrap().write(self.buf.as_slice()));
        }

        Ok(())
    }

    /// Unwrap the underlying writer, finishing the compression stream.
    pub fn into_inner(mut self) -> Result<W, (BzCompressor<W>, IoError)> {
        match self.do_write(&[], Action::Finish) {
            Ok(()) => {}
            Err(e) => return Err((self, e)),
        }
        Ok(self.w.take().unwrap())
    }
}

impl<W: Writer> Writer for BzCompressor<W> {
    fn write(&mut self, data: &[u8]) -> IoResult<()> {
        self.do_write(data, Action::Run)
    }

    fn flush(&mut self) -> IoResult<()> {
        try!(self.do_write(&[], Action::Flush));
        self.w.as_mut().unwrap().flush()
    }
}

#[unsafe_destructor]
impl<W: Writer> Drop for BzCompressor<W> {
    fn drop(&mut self) {
        if self.w.is_some() {
            let _ = self.do_write(&[], Action::Finish);
        }
    }
}

impl<W: Writer> BzDecompressor<W> {
    /// Create a new compression stream which will compress at the given level
    /// to write compress output to the give output stream.
    pub fn new(w: W) -> BzDecompressor<W> {
        BzDecompressor {
            stream: Stream::new_decompress(false),
            w: Some(w),
            buf: Vec::with_capacity(128 * 1024),
            done: false,
        }
    }

    fn do_write(&mut self, mut data: &[u8], action: Action) -> IoResult<()> {
        while data.len() > 0 || (action == Action::Finish && !self.done) {
            let total_in = self.stream.total_in();
            let rc = self.stream.decompress_vec(data, &mut self.buf);
            data = &data[(self.stream.total_in() - total_in) as usize..];

            if self.buf.len() == self.buf.capacity() {
                try!(self.w.as_mut().unwrap().write(self.buf.as_slice()));
                self.buf.truncate(0);
            }

            match rc {
                ffi::BZ_STREAM_END => { self.done = true; break }
                n if n >= 0 => {}
                ffi::BZ_OUTBUFF_FULL => {}
                n => panic!("unexpected return: {}", n),
            }
        }

        if action == Action::Finish {
            try!(self.w.as_mut().unwrap().write(self.buf.as_slice()));
        }

        Ok(())
    }

    /// Unwrap the underlying writer, finishing the compression stream.
    pub fn into_inner(mut self) -> Result<W, (BzDecompressor<W>, IoError)> {
        match self.do_write(&[], Action::Finish) {
            Ok(()) => {}
            Err(e) => return Err((self, e)),
        }
        Ok(self.w.take().unwrap())
    }
}

impl<W: Writer> Writer for BzDecompressor<W> {
    fn write(&mut self, data: &[u8]) -> IoResult<()> {
        self.do_write(data, Action::Run)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.w.as_mut().unwrap().flush()
    }
}

#[unsafe_destructor]
impl<W: Writer> Drop for BzDecompressor<W> {
    fn drop(&mut self) {
        if self.w.is_some() {
            let _ = self.do_write(&[], Action::Finish);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::MemWriter;
    use std::iter::repeat;
    use super::{BzCompressor, BzDecompressor};

    #[test]
    fn smoke() {
        let d = BzDecompressor::new(MemWriter::new());
        let mut c = BzCompressor::new(d, ::CompressionLevel::Default);
        c.write(b"12834").unwrap();
        let s = repeat("12345").take(100000).collect::<String>();
        c.write(s.as_bytes()).unwrap();
        let data = c.into_inner().ok().unwrap()
                    .into_inner().ok().unwrap().into_inner();
        assert_eq!(&data[0..5], b"12834");
        assert_eq!(data.len(), 500005);
        assert!(format!("12834{}", s).as_bytes() == data.as_slice());
    }
}
