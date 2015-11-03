//! Writer-based compression/decompression streams

use std::io::prelude::*;
use std::io;

use ffi;
use raw::{Stream, Action};

/// A compression stream which will have uncompressed data written to it and
/// will write compressed data to an output stream.
pub struct BzCompressor<W: Write> {
    stream: Stream,
    w: Option<W>,
    buf: Vec<u8>,
}

/// A compression stream which will have compressed data written to it and
/// will write uncompressed data to an output stream.
pub struct BzDecompressor<W: Write> {
    stream: Stream,
    w: Option<W>,
    buf: Vec<u8>,
    done: bool,
}

impl<W: Write> BzCompressor<W> {
    /// Create a new compression stream which will compress at the given level
    /// to write compress output to the give output stream.
    pub fn new(w: W, level: ::Compress) -> BzCompressor<W> {
        BzCompressor {
            stream: Stream::new_compress(level, 30),
            w: Some(w),
            buf: Vec::with_capacity(128 * 1024),
        }
    }

    fn do_write(&mut self, data: &[u8], action: Action) -> io::Result<usize> {
        if self.buf.len() > 0 {
            try!(self.w.as_mut().unwrap().write_all(&self.buf));
            self.buf.truncate(0);
        }

        let total_in = self.stream.total_in();
        let rc = self.stream.compress_vec(data, &mut self.buf, action);
        let written = (self.stream.total_in() - total_in) as usize;

        if rc < 0 {
            panic!("unexpected return: {}", rc);
        }

        if action == Action::Finish && self.buf.len() > 0 {
            try!(self.w.as_mut().unwrap().write_all(&self.buf));
            self.buf.truncate(0);
        }
        Ok(written)
    }

    /// Unwrap the underlying writer, finishing the compression stream.
    pub fn into_inner(mut self) -> Result<W, (BzCompressor<W>, io::Error)> {
        match self.do_write(&[], Action::Finish) {
            Ok(_) => {}
            Err(e) => return Err((self, e)),
        }
        Ok(self.w.take().unwrap())
    }

    /// Returns the number of bytes produced by the compressor
    ///
    /// Note that, due to buffering, this only bears any relation to
    /// `total_in()` after a call to `flush()`.  At that point,
    /// `total_out() / total_in()` is the compression ratio.
    pub fn total_out(&self) -> u64 {
        self.stream.total_out()
    }

    /// Returns the number of bytes consumed by the compressor
    /// (e.g. the number of bytes written to this stream.)
    pub fn total_in(&self) -> u64 {
        self.stream.total_in()
    }
}

impl<W: Write> Write for BzCompressor<W> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if data.len() != 0 {
            self.do_write(data, Action::Run)
        } else {
            Ok(0)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        try!(self.do_write(&[], Action::Flush));
        self.w.as_mut().unwrap().flush()
    }
}

impl<W: Write> Drop for BzCompressor<W> {
    fn drop(&mut self) {
        if self.w.is_some() {
            let _ = self.do_write(&[], Action::Finish);
        }
    }
}

impl<W: Write> BzDecompressor<W> {
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

    fn do_write(&mut self, data: &[u8], action: Action) -> io::Result<usize> {
        loop {
            if self.buf.len() > 0 {
                try!(self.w.as_mut().unwrap().write_all(&self.buf));
                self.buf.truncate(0);
            }

            let (written, rc) = if self.done {(0, 0)} else {
                let total_in = self.stream.total_in();
                let rc = self.stream.decompress_vec(data, &mut self.buf);
                ((self.stream.total_in() - total_in) as usize, rc)
            };

            match rc {
                ffi::BZ_STREAM_END => self.done = true,
                n if n >= 0 => {}
                n => panic!("unexpected return: {}", n),
            }

            match action {
                Action::Run if written == 0 => continue,
                Action::Finish if self.buf.len() > 0 => {
                    try!(self.w.as_mut().unwrap().write_all(&self.buf));
                    self.buf.truncate(0);
                }
                _ => {}
            }

            return Ok(written)
        }
    }

    /// Unwrap the underlying writer, finishing the compression stream.
    pub fn into_inner(mut self) -> Result<W, (BzDecompressor<W>, io::Error)> {
        match self.do_write(&[], Action::Finish) {
            Ok(_) => {}
            Err(e) => return Err((self, e)),
        }
        Ok(self.w.take().unwrap())
    }

    /// Returns the number of bytes produced by the decompressor
    ///
    /// Note that, due to buffering, this only bears any relation to
    /// `total_in()` after a call to `flush()`.  At that point,
    /// `total_in() / total_out()` is the compression ratio.
    pub fn total_out(&self) -> u64 {
        self.stream.total_out()
    }

    /// Returns the number of bytes consumed by the decompressor
    /// (e.g. the number of bytes written to this stream.)
    pub fn total_in(&self) -> u64 {
        self.stream.total_in()
    }
}

impl<W: Write> Write for BzDecompressor<W> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.do_write(data, Action::Run)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w.as_mut().unwrap().flush()
    }
}

impl<W: Write> Drop for BzDecompressor<W> {
    fn drop(&mut self) {
        if self.w.is_some() {
            let _ = self.do_write(&[], Action::Finish);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::iter::repeat;
    use super::{BzCompressor, BzDecompressor};

    #[test]
    fn smoke() {
        let d = BzDecompressor::new(Vec::new());
        let mut c = BzCompressor::new(d, ::Compress::Default);
        c.write_all(b"12834").unwrap();
        let s = repeat("12345").take(100000).collect::<String>();
        c.write_all(s.as_bytes()).unwrap();
        let data = c.into_inner().ok().unwrap()
                    .into_inner().ok().unwrap();
        assert_eq!(&data[0..5], b"12834");
        assert_eq!(data.len(), 500005);
        assert!(format!("12834{}", s).as_bytes() == &*data);
    }

    #[test]
    fn write_empty() {
        let d = BzDecompressor::new(Vec::new());
        let mut c = BzCompressor::new(d, ::Compress::Default);
        c.write(b"").unwrap();
        let data = c.into_inner().ok().unwrap()
                    .into_inner().ok().unwrap();
        assert_eq!(&data[..], b"");
    }
}
