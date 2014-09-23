//! Raw un-exported bindings to miniz for encoding/decoding

use std::io;
use std::mem;
use std::io::IoResult;
use libc;

use {CompressionLevel, NoCompression};
use ffi;

pub struct EncoderWriter<W> {
    pub inner: Option<W>,
    stream: Stream,
    buf: Vec<u8>,
}

pub struct EncoderReader<R> {
    pub inner: R,
    stream: Stream,
    buf: Vec<u8>,
    pos: uint,
}

pub struct DecoderReader<R> {
    pub inner: R,
    stream: Stream,
    pos: uint,
    pub buf: Vec<u8>,
}

pub struct DecoderWriter<W> {
    pub inner: Option<W>,
    stream: Stream,
    buf: Vec<u8>,
}

enum Flavor { Compress, Decompress }

struct Stream(ffi::bz_stream, Flavor);

impl<W: Writer> EncoderWriter<W> {
    pub fn new(w: W, level: CompressionLevel, raw: bool,
               buf: Vec<u8>) -> EncoderWriter<W> {
        EncoderWriter {
            inner: Some(w),
            stream: Stream::new(Compress, raw, level),
            buf: buf,
        }
    }

    pub fn do_finish(&mut self) -> IoResult<()> {
        try!(self.stream.write([], ffi::BZ_FINISH, &mut self.buf,
                               self.inner.as_mut().unwrap(), ffi::bz_deflate));
        try!(self.inner.as_mut().unwrap().write(self.buf.as_slice()));
        self.buf.truncate(0);
        Ok(())
    }
}

impl<W: Writer> Writer for EncoderWriter<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.stream.write(buf, ffi::BZ_NO_FLUSH, &mut self.buf,
                          self.inner.as_mut().unwrap(), ffi::bz_deflate)
    }

    fn flush(&mut self) -> IoResult<()> {
        let inner = self.inner.as_mut().unwrap();
        try!(self.stream.write([], ffi::BZ_SYNC_FLUSH, &mut self.buf, inner,
                               ffi::bz_deflate));
        inner.flush()
    }
}

#[unsafe_destructor]
impl<W: Writer> Drop for EncoderWriter<W> {
    fn drop(&mut self) {
        match self.inner {
            Some(..) => { let _ = self.do_finish(); }
            None => {}
        }
    }
}

impl<R: Reader> EncoderReader<R> {
    pub fn new(w: R, level: CompressionLevel, raw: bool,
               buf: Vec<u8>) -> EncoderReader<R> {
        EncoderReader {
            inner: w,
            stream: Stream::new(Compress, raw, level),
            buf: buf,
            pos: 0,
        }
    }
}

impl<R: Reader> Reader for EncoderReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
        self.stream.read(buf, &mut self.buf, &mut self.pos,
                         &mut self.inner, ffi::bz_deflate)
    }
}

impl<R: Reader> DecoderReader<R> {
    pub fn new(r: R, raw: bool, buf: Vec<u8>) -> DecoderReader<R> {
        DecoderReader {
            inner: r,
            stream: Stream::new(Decompress, raw, NoCompression),
            pos: 0,
            buf: buf,
        }
    }
}

impl<R: Reader> Reader for DecoderReader<R> {
    fn read(&mut self, into: &mut [u8]) -> IoResult<uint> {
        self.stream.read(into, &mut self.buf, &mut self.pos,
                         &mut self.inner, ffi::bz_inflate)
    }
}

impl<W: Writer> DecoderWriter<W> {
    pub fn new(w: W, raw: bool, buf: Vec<u8>) -> DecoderWriter<W> {
        DecoderWriter {
            inner: Some(w),
            stream: Stream::new(Decompress, raw, NoCompression),
            buf: buf,
        }
    }

    pub fn do_finish(&mut self) -> IoResult<()> {
        try!(self.stream.write([], ffi::BZ_FINISH, &mut self.buf,
                               self.inner.as_mut().unwrap(), ffi::bz_inflate));
        try!(self.inner.as_mut().unwrap().write(self.buf.as_slice()));
        self.buf.truncate(0);
        Ok(())
    }
}

impl<W: Writer> Writer for DecoderWriter<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        self.stream.write(buf, ffi::BZ_NO_FLUSH, &mut self.buf,
                          self.inner.as_mut().unwrap(), ffi::bz_inflate)
    }

    fn flush(&mut self) -> IoResult<()> {
        let inner = self.inner.as_mut().unwrap();
        try!(self.stream.write([], ffi::BZ_SYNC_FLUSH, &mut self.buf, inner,
                               ffi::bz_inflate));
        inner.flush()
    }
}

impl Stream {
    fn new(kind: Flavor, raw: bool, level: CompressionLevel) -> Stream {
        let mut state: ffi::bz_stream = unsafe { mem::zeroed() };
        let ret = match kind {
            Compress => unsafe {
                ffi::BZ2_bzCompressInit(&mut state, level as libc::c_int, 0, 30)
            },
            Decompress => unsafe {
                ffi::BZ2_bzDecompressInit(&mut state, 0, 0)
            }
        };
        assert_eq!(ret, 0);
        Stream(state, kind)
    }

    fn read<R: Reader>(&mut self, into: &mut [u8], buf: &mut Vec<u8>,
                       pos: &mut uint, reader: &mut R,
                       f: unsafe extern fn(*mut ffi::bz_stream,
                                           libc::c_int) -> libc::c_int)
                       -> IoResult<uint> {

        let cap = buf.capacity();
        let mut read = 0;
        let mut eof = false;
        while read < into.len() {
            if *pos == buf.len() {
                buf.truncate(0);
                match reader.push(cap, buf) {
                    Ok(..) => {}
                    Err(ref e) if e.kind == io::EndOfFile => eof = true,
                    Err(e) => return Err(e),
                }
                *pos = 0;
            }

            self.next_in = buf.slice_from(*pos).as_ptr();
            self.avail_in = (buf.len() - *pos) as libc::c_uint;
            self.next_out = into.slice_from_mut(read).as_mut_ptr();
            self.avail_out = (into.len() - read) as libc::c_uint;

            let before_out = self.total_out;
            let before_in = self.total_in;

            let flush = if eof {ffi::BZ_FINISH} else {ffi::BZ_NO_FLUSH};
            let ret = unsafe { f(&mut **self, flush) };
            read += (self.total_out - before_out) as uint;
            *pos += (self.total_in - before_in) as uint;

            match ret {
                ffi::BZ_OK => {}
                ffi::BZ_STREAM_END if read > 0 => break,
                ffi::BZ_STREAM_END => {
                    return Err(io::standard_error(io::EndOfFile))
                }
                ffi::BZ_OUTBUFF_FULL => break,
                ffi::BZ_DATA_ERROR => {
                    return Err(io::standard_error(io::InvalidInput))
                }
                n => fail!("unexpected return {}", n),
            }
        }

        Ok(read)
    }

    fn write<W: Writer>(&mut self, mut buf: &[u8], flush: libc::c_int,
                        into: &mut Vec<u8>, writer: &mut W,
                        f: unsafe extern fn(*mut ffi::bz_stream,
                                            libc::c_int) -> libc::c_int)
                        -> IoResult<()> {
        let cap = into.capacity();
        while buf.len() > 0 || flush == ffi::BZ_FINISH {
            self.next_in = buf.as_ptr();
            self.avail_in = buf.len() as libc::c_uint;
            let cur_len = into.len();
            self.next_out = into.slice_from_mut(cur_len).as_mut_ptr();
            self.avail_out = (cap - cur_len) as libc::c_uint;

            let before_out = self.total_out;
            let before_in = self.total_in;

            let ret = unsafe {
                let ret = f(&mut **self, flush);
                into.set_len(cur_len + (self.total_out - before_out) as uint);
                ret
            };
            buf = buf.slice_from((self.total_in - before_in) as uint);

            if cap - cur_len == 0 || ret == ffi::BZ_OUTBUFF_FULL {
                try!(writer.write(into.as_slice()));
                into.truncate(0);
            }
            match ret {
                ffi::BZ_OK => {},
                ffi::BZ_STREAM_END => return Ok(()),
                ffi::BZ_OUTBUFF_FULL => {}
                n => fail!("unexpected return {}", n),
            }
        }

        Ok(())
    }
}

impl Deref<ffi::bz_stream> for Stream {
    fn deref<'a>(&'a self) -> &'a ffi::bz_stream {
        let Stream(ref inner, _) = *self; inner
    }
}

impl DerefMut<ffi::bz_stream> for Stream {
    fn deref_mut<'a>(&'a mut self) -> &'a mut ffi::bz_stream {
        let Stream(ref mut inner, _) = *self; inner
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        unsafe {
            match *self {
                Stream(ref mut s, Compress) => ffi::BZ2_bzCompressEnd(s),
                Stream(ref mut s, Decompress) => ffi::BZ2_bzDecompressEnd(s),
            };
        }
    }
}
