//! Reader-based compression/decompression streams

use std::io::{self, IoResult};

use ffi;
use raw::{Stream, Action};

/// A compression stream which wraps an uncompressed stream of data. Compressed
/// data will be read from the stream.
pub struct BzCompressor<R> {
    stream: Stream,
    r: R,
    buf: Vec<u8>,
    pos: uint,
    done: bool,
}

/// A decompression stream which wraps a compressed stream of data. Decompressed
/// data will be read from the stream.
pub struct BzDecompressor<R> {
    stream: Stream,
    r: R,
    buf: Vec<u8>,
    pos: uint,
    done: bool,
}

impl<R: Reader> BzCompressor<R> {
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

impl<R: Reader> Reader for BzCompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        if self.done { return Err(io::standard_error(io::EndOfFile)) }

        let mut read = 0;
        let cap = self.buf.capacity();
        let mut eof = false;
        while read < buf.len() {
            if self.pos == self.buf.len() {
                self.buf.truncate(0);
                match self.r.push(cap, &mut self.buf) {
                    Ok(..) => {}
                    Err(ref e) if e.kind == io::EndOfFile => eof = true,
                    Err(e) => return Err(e),
                }
                self.pos = 0;
            }

            let before_in = self.stream.total_in();
            let before_out = self.stream.total_out();
            let action = if eof {Action::Finish} else {Action::Run};
            let rc = self.stream.compress(self.buf.slice_from(self.pos),
                                          buf.slice_from_mut(read),
                                          action);
            self.pos += (self.stream.total_in() - before_in) as uint;
            read += (self.stream.total_out() - before_out) as uint;

            match rc {
                ffi::BZ_STREAM_END if read > 0 => { self.done = true; break }
                ffi::BZ_OUTBUFF_FULL |
                ffi::BZ_STREAM_END => {
                    return Err(io::standard_error(io::EndOfFile))
                }

                n if n >= 0 => {}
                _ => return Err(io::standard_error(io::InvalidInput)),
            }
        }

        Ok(read)
    }
}

impl<R: Reader> BzDecompressor<R> {
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

impl<R: Reader> Reader for BzDecompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        if self.done { return Err(io::standard_error(io::EndOfFile)) }

        let mut read = 0;
        let cap = self.buf.capacity();
        while read < buf.len() {
            if self.pos == self.buf.len() {
                self.buf.truncate(0);
                match self.r.push(cap, &mut self.buf) {
                    Ok(..) => {}
                    Err(ref e) if e.kind == io::EndOfFile => {}
                    Err(e) => return Err(e),
                }
                self.pos = 0;
            }

            let before_in = self.stream.total_in();
            let before_out = self.stream.total_out();
            let rc = self.stream.decompress(self.buf.slice_from(self.pos),
                                            buf.slice_from_mut(read));
            self.pos += (self.stream.total_in() - before_in) as uint;
            read += (self.stream.total_out() - before_out) as uint;

            match rc {
                ffi::BZ_STREAM_END if read > 0 => { self.done = true; break }
                ffi::BZ_OUTBUFF_FULL |
                ffi::BZ_STREAM_END => {
                    return Err(io::standard_error(io::EndOfFile))
                }

                n if n >= 0 => {}
                _ => return Err(io::standard_error(io::InvalidInput)),
            }
        }

        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{MemReader, MemWriter};
    use super::{BzCompressor, BzDecompressor};
    use writer as w;

    #[test]
    fn smoke() {
        let m = MemReader::new(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let mut c = BzCompressor::new(m, ::CompressionLevel::Default);
        let data = c.read_to_end().unwrap();
        let mut d = w::BzDecompressor::new(MemWriter::new());
        d.write(data.as_slice()).unwrap();
        assert_eq!(d.into_inner().ok().unwrap().into_inner().as_slice(),
                   [1, 2, 3, 4, 5, 6, 7, 8].as_slice());
    }

    #[test]
    fn smoke2() {
        let m = MemReader::new(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let c = BzCompressor::new(m, ::CompressionLevel::Default);
        let mut d = BzDecompressor::new(c);
        let data = d.read_to_end().unwrap();
        assert_eq!(data.as_slice(),
                   [1, 2, 3, 4, 5, 6, 7, 8].as_slice());
    }
}

