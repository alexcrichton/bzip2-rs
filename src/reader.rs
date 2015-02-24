//! Reader-based compression/decompression streams

use std::io;
use std::io::prelude::*;

use ffi;
use raw::{Stream, Action};

/// A compression stream which wraps an uncompressed stream of data. Compressed
/// data will be read from the stream.
pub struct BzCompressor<R> {
    stream: Stream,
    r: R,
    buf: Vec<u8>,
    pos: usize,
    done: bool,
}

/// A decompression stream which wraps a compressed stream of data. Decompressed
/// data will be read from the stream.
pub struct BzDecompressor<R> {
    stream: Stream,
    r: R,
    buf: Vec<u8>,
    pos: usize,
    done: bool,
}

impl<R: Read> BzCompressor<R> {
    /// Create a new compression stream which will compress at the given level
    /// to read compress output to the give output stream.
    pub fn new(r: R, level: ::CompressionLevel) -> BzCompressor<R> {
        BzCompressor {
            stream: Stream::new_compress(level, 30),
            r: r,
            buf: Vec::with_capacity(128 * 1024),
            pos: 0,
            done: false,
        }
    }

    /// Unwrap the underlying writer, finishing the compression stream.
    pub fn into_inner(self) -> R { self.r }
}

impl<R: Read> Read for BzCompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.done { return Ok(0) }

        let mut read = 0;
        let cap = self.buf.capacity() as u64;
        let mut eof = false;
        while read < buf.len() {
            if self.pos == self.buf.len() {
                self.buf.truncate(0);
                match self.r.by_ref().take(cap).read_to_end(&mut self.buf) {
                    Ok(..) if self.buf.len() < cap as usize => { eof = true; },
                    Ok(..) => {}
                    Err(e) => return Err(e),
                }
                self.pos = 0;
            }

            let before_in = self.stream.total_in();
            let before_out = self.stream.total_out();
            let action = if eof {Action::Finish} else {Action::Run};
            let rc = self.stream.compress(&self.buf[self.pos..],
                                          &mut buf[read..],
                                          action);
            self.pos += (self.stream.total_in() - before_in) as usize;
            read += (self.stream.total_out() - before_out) as usize;

            match rc {
                ffi::BZ_STREAM_END if read > 0 => { self.done = true; break }
                ffi::BZ_OUTBUFF_FULL |
                ffi::BZ_STREAM_END => {
                    return Ok(0)
                }

                n if n >= 0 => {}
                _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid input", None)),
            }
        }

        Ok(read)
    }
}

impl<R: Read> BzDecompressor<R> {
    /// Create a new compression stream which will compress at the given level
    /// to read compress output to the give output stream.
    pub fn new(r: R) -> BzDecompressor<R> {
        BzDecompressor {
            stream: Stream::new_decompress(false),
            r: r,
            buf: Vec::with_capacity(128 * 1024),
            done: false,
            pos: 0,
        }
    }

    /// Unwrap the underlying writer, finishing the compression stream.
    pub fn into_inner(self) -> R { self.r }
}

impl<R: Read> Read for BzDecompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.done { return Ok(0) }

        let mut read = 0;
        let cap = self.buf.capacity() as u64;
        while read < buf.len() {
            if self.pos == self.buf.len() {
                self.buf.truncate(0);
                match self.r.by_ref().take(cap).read_to_end(&mut self.buf) {
                    Ok(..) => {}
                    Err(e) => return Err(e),
                }
                self.pos = 0;
            }

            let before_in = self.stream.total_in();
            let before_out = self.stream.total_out();
            let rc = self.stream.decompress(&self.buf[self.pos..],
                                            &mut buf[read..]);
            self.pos += (self.stream.total_in() - before_in) as usize;
            read += (self.stream.total_out() - before_out) as usize;

            match rc {
                ffi::BZ_STREAM_END if read > 0 => { self.done = true; break }
                ffi::BZ_OUTBUFF_FULL |
                ffi::BZ_STREAM_END => {
                    return Ok(0)
                }

                n if n >= 0 => {}
                _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid input", None)),
            }
        }

        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use super::{BzCompressor, BzDecompressor};
    use writer as w;

    #[test]
    fn smoke() {
        let m: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8];
        let mut c = BzCompressor::new(m, ::CompressionLevel::Default);
        let mut data = vec![];
        c.read_to_end(&mut data).unwrap();
        let mut d = w::BzDecompressor::new(vec![]);
        d.write_all(data.as_slice()).unwrap();
        assert_eq!(&d.into_inner().ok().unwrap(),
                   &[1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn smoke2() {
        let m: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8];
        let c = BzCompressor::new(m, ::CompressionLevel::Default);
        let mut d = BzDecompressor::new(c);
        let mut data = vec![];
        d.read_to_end(&mut data).unwrap();
        assert_eq!(data.as_slice(),
                   [1, 2, 3, 4, 5, 6, 7, 8].as_slice());
    }
}

