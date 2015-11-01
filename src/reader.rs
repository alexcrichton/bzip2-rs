//! Reader-based compression/decompression streams

use std::io::prelude::*;
use std::io;
use std::iter::repeat;
use libc::c_int;

use ffi;
use raw::{Stream, Action};

/// A compression stream which wraps an uncompressed stream of data. Compressed
/// data will be read from the stream.
pub struct BzCompressor<R>(Inner<R>);

/// A decompression stream which wraps a compressed stream of data. Decompressed
/// data will be read from the stream.
pub struct BzDecompressor<R>(Inner<R>);

struct Inner<R> {
    stream: Stream,
    r: R,
    buf: Vec<u8>,
    cap: usize,
    pos: usize,
    done: bool,
}

impl<R: Read> BzCompressor<R> {
    /// Create a new compression stream which will compress at the given level
    /// to read compress output to the give output stream.
    pub fn new(r: R, level: ::Compress) -> BzCompressor<R> {
        BzCompressor(Inner {
            stream: Stream::new_compress(level, 30),
            r: r,
            buf: repeat(0).take(32 * 1024).collect(),
            cap: 0,
            pos: 0,
            done: false,
        })
    }

    /// Unwrap the underlying writer, finishing the compression stream.
    pub fn into_inner(self) -> R { self.0.r }
}

impl<R: Read> Read for BzCompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf.len(), |stream, input, offset, eof| {
            let action = if eof {Action::Finish} else {Action::Run};
            stream.compress(input, &mut buf[offset..], action)
        })
    }
}

impl<R: Read> BzDecompressor<R> {
    /// Create a new compression stream which will compress at the given level
    /// to read compress output to the give output stream.
    pub fn new(r: R) -> BzDecompressor<R> {
        BzDecompressor(Inner {
            stream: Stream::new_decompress(false),
            r: r,
            buf: repeat(0).take(32 * 1024).collect::<Vec<_>>(),
            cap: 0,
            done: false,
            pos: 0,
        })
    }

    /// Unwrap the underlying writer, finishing the compression stream.
    pub fn into_inner(self) -> R { self.0.r }
}

impl<R: Read> Read for BzDecompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf.len(), |stream, input, offset, _eof| {
            stream.decompress(input, &mut buf[offset..])
        })
    }
}

impl<R: Read> Inner<R> {
    fn read<F>(&mut self, target: usize, mut f: F) -> io::Result<usize>
        where F: FnMut(&mut Stream, &[u8], usize, bool) -> c_int
    {
        if self.done { return Ok(0) }

        let mut read = 0;

        loop {
            let mut eof = false;
            if self.pos == self.cap {
                self.cap = try!(self.r.read(&mut self.buf));
                self.pos = 0;
                eof = self.cap == 0;
            }
            let before_in = self.stream.total_in();
            let before_out = self.stream.total_out();
            let rc = f(&mut self.stream, &self.buf[self.pos..self.cap], read, eof);
            self.pos += (self.stream.total_in() - before_in) as usize;
            read += (self.stream.total_out() - before_out) as usize;

            match rc {
                ffi::BZ_STREAM_END => { self.done = true; eof = true; }
                ffi::BZ_OUTBUFF_FULL => {}
                n if n >= 0 => {}

                _ => return Err(io::Error::new(io::ErrorKind::InvalidInput,
                                               "invalid input")),
            }
            if target > read && !eof { continue }
            return Ok(read)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use super::{BzCompressor, BzDecompressor};
    use writer as w;
    use rand::{thread_rng, Rng};

    #[test]
    fn smoke() {
        let m: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8];
        let mut c = BzCompressor::new(m, ::Compress::Default);
        let mut data = vec![];
        c.read_to_end(&mut data).unwrap();
        let mut d = w::BzDecompressor::new(vec![]);
        d.write_all(&data).unwrap();
        assert_eq!(&d.into_inner().ok().unwrap(),
                   &[1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn smoke2() {
        let m: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8];
        let c = BzCompressor::new(m, ::Compress::Default);
        let mut d = BzDecompressor::new(c);
        let mut data = vec![];
        d.read_to_end(&mut data).unwrap();
        assert_eq!(data, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn smoke3() {
        let m = vec![3u8; 128 * 1024 + 1];
        let c = BzCompressor::new(&m[..], ::Compress::Default);
        let mut d = BzDecompressor::new(c);
        let mut data = vec![];
        d.read_to_end(&mut data).unwrap();
        assert!(data == &m[..]);
    }

    #[test]
    fn self_terminating() {
        let m = vec![3u8; 128 * 1024 + 1];
        let mut c = BzCompressor::new(&m[..], ::Compress::Default);

        let mut result = Vec::new();
        c.read_to_end(&mut result).unwrap();

        let v = thread_rng().gen_iter::<u8>().take(1024).collect::<Vec<_>>();
        for _ in 0..200 {
            result.extend(v.iter().map(|x| *x));
        }

        let mut d = BzDecompressor::new(&result[..]);
        let mut data = Vec::with_capacity(m.len());
        unsafe { data.set_len(m.len()); }
        assert!(d.read(&mut data).unwrap() == m.len());
        assert!(data == &m[..]);
    }

    #[test]
    fn zero_length_read_at_eof() {
        let m = Vec::new();
        let mut c = BzCompressor::new(&m[..], ::Compress::Default);

        let mut result = Vec::new();
        c.read_to_end(&mut result).unwrap();

        let mut d = BzDecompressor::new(&result[..]);
        let mut data = Vec::new();
        assert!(d.read(&mut data).unwrap() == 0);
    }

    #[test]
    fn zero_length_read_with_data() {
        let m = vec![3u8; 128 * 1024 + 1];
        let mut c = BzCompressor::new(&m[..], ::Compress::Default);

        let mut result = Vec::new();
        c.read_to_end(&mut result).unwrap();

        let mut d = BzDecompressor::new(&result[..]);
        let mut data = Vec::new();
        assert!(d.read(&mut data).unwrap() == 0);
    }
}
