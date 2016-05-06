//! Writer-based compression/decompression streams

use std::io::prelude::*;
use std::io;

use {Action, Status, Compression, Compress, Decompress};

/// A compression stream which will have uncompressed data written to it and
/// will write compressed data to an output stream.
pub struct BzEncoder<W: Write> {
    data: Compress,
    obj: Option<W>,
    buf: Vec<u8>,
}

/// A compression stream which will have compressed data written to it and
/// will write uncompressed data to an output stream.
pub struct BzDecoder<W: Write> {
    data: Decompress,
    obj: Option<W>,
    buf: Vec<u8>,
    done: bool,
}

impl<W: Write> BzEncoder<W> {
    /// Create a new compression stream which will compress at the given level
    /// to write compress output to the give output stream.
    pub fn new(obj: W, level: Compression) -> BzEncoder<W> {
        BzEncoder {
            data: Compress::new(level, 30),
            obj: Some(obj),
            buf: Vec::with_capacity(32 * 1024),
        }
    }

    fn dump(&mut self) -> io::Result<()> {
        if self.buf.len() > 0 {
            try!(self.obj.as_mut().unwrap().write_all(&self.buf));
            self.buf.truncate(0);
        }
        Ok(())
    }

    fn do_finish(&mut self) -> io::Result<()> {
        loop {
            try!(self.dump());
            let res = self.data.compress_vec(&[], &mut self.buf, Action::Finish);
            if res == Ok(Status::StreamEnd) {
                break
            }
        }
        self.dump()
    }

    /// Consumes this encoder, flushing the output stream.
    ///
    /// This will flush the underlying data stream and then return the contained
    /// writer if the flush succeeded.
    pub fn finish(mut self) -> io::Result<W> {
        try!(self.do_finish());
        Ok(self.obj.take().unwrap())
    }

    /// Returns the number of bytes produced by the compressor
    ///
    /// Note that, due to buffering, this only bears any relation to
    /// `total_in()` after a call to `flush()`.  At that point,
    /// `total_out() / total_in()` is the compression ratio.
    pub fn total_out(&self) -> u64 {
        self.data.total_out()
    }

    /// Returns the number of bytes consumed by the compressor
    /// (e.g. the number of bytes written to this stream.)
    pub fn total_in(&self) -> u64 {
        self.data.total_in()
    }
}

impl<W: Write> Write for BzEncoder<W> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        loop {
            try!(self.dump());

            let total_in = self.total_in();
            self.data.compress_vec(data, &mut self.buf, Action::Run)
                .unwrap();
            let written = (self.total_in() - total_in) as usize;

            if written > 0 || data.len() == 0 {
                return Ok(written)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        loop {
            try!(self.dump());
            let before = self.total_out();
            self.data.compress_vec(&[], &mut self.buf, Action::Flush)
                .unwrap();

            if before == self.total_out() {
                break
            }
        }
        self.obj.as_mut().unwrap().flush()
    }
}

impl<W: Write> Drop for BzEncoder<W> {
    fn drop(&mut self) {
        if self.obj.is_some() {
            let _ = self.do_finish();
        }
    }
}

impl<W: Write> BzDecoder<W> {
    /// Create a new decoding stream which will decompress all data written
    /// to it into `obj`.
    pub fn new(obj: W) -> BzDecoder<W> {
        BzDecoder {
            data: Decompress::new(false),
            obj: Some(obj),
            buf: Vec::with_capacity(32 * 1024),
            done: false,
        }
    }

    fn dump(&mut self) -> io::Result<()> {
        if self.buf.len() > 0 {
            try!(self.obj.as_mut().unwrap().write_all(&self.buf));
            self.buf.truncate(0);
        }
        Ok(())
    }

    fn do_finish(&mut self) -> io::Result<()> {
        while !self.done {
            try!(self.write(&[]));
        }
        self.dump()
    }

    /// Unwrap the underlying writer, finishing the compression stream.
    pub fn finish(&mut self) -> io::Result<W> {
        try!(self.do_finish());
        Ok(self.obj.take().unwrap())
    }

    /// Returns the number of bytes produced by the decompressor
    ///
    /// Note that, due to buffering, this only bears any relation to
    /// `total_in()` after a call to `flush()`.  At that point,
    /// `total_in() / total_out()` is the compression ratio.
    pub fn total_out(&self) -> u64 {
        self.data.total_out()
    }

    /// Returns the number of bytes consumed by the decompressor
    /// (e.g. the number of bytes written to this stream.)
    pub fn total_in(&self) -> u64 {
        self.data.total_in()
    }
}

impl<W: Write> Write for BzDecoder<W> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.done {
            return Ok(0)
        }
        loop {
            try!(self.dump());

            let before = self.total_in();
            let res = self.data.decompress_vec(data, &mut self.buf);
            let written = (self.total_in() - before) as usize;

            let res = try!(res.map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidInput, e)
            }));

            if res == Status::StreamEnd {
                self.done = true;
            }
            if written > 0 || data.len() == 0 || self.done {
                return Ok(written)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        try!(self.dump());
        self.obj.as_mut().unwrap().flush()
    }
}

impl<W: Write> Drop for BzDecoder<W> {
    fn drop(&mut self) {
        if self.obj.is_some() {
            let _ = self.do_finish();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::iter::repeat;
    use super::{BzEncoder, BzDecoder};

    #[test]
    fn smoke() {
        let d = BzDecoder::new(Vec::new());
        let mut c = BzEncoder::new(d, ::Compression::Default);
        c.write_all(b"12834").unwrap();
        let s = repeat("12345").take(100000).collect::<String>();
        c.write_all(s.as_bytes()).unwrap();
        let data = c.finish().unwrap().finish().unwrap();
        assert_eq!(&data[0..5], b"12834");
        assert_eq!(data.len(), 500005);
        assert!(format!("12834{}", s).as_bytes() == &*data);
    }

    #[test]
    fn write_empty() {
        let d = BzDecoder::new(Vec::new());
        let mut c = BzEncoder::new(d, ::Compression::Default);
        c.write(b"").unwrap();
        let data = c.finish().unwrap().finish().unwrap();
        assert_eq!(&data[..], b"");
    }
}
