//! Raw low-level manipulations of bz streams.

use std::mem;
use libc::{c_int, c_uint};

use ffi;

/// Wrapper around a raw instance of `bz_stream`.
pub struct Stream {
    // libbz2 requires a stable address for this stream.
    raw: Box<ffi::bz_stream>,
    kind: Kind,
}

/// Kinds of streams
#[derive(Copy)]
pub enum Kind {
    /// Streams used for compression
    Compress,
    /// Streams used for decompression
    Decompress,
}

/// Possible actions to take on compression.
#[derive(PartialEq, Eq, Copy, Debug)]
pub enum Action {
    /// Normal compression.
    Run = ffi::BZ_RUN as isize,
    /// Request that the current compression block is terminate.
    Flush = ffi::BZ_FLUSH as isize,
    /// Request that the compression stream be finalized.
    Finish = ffi::BZ_FINISH as isize,
}

impl Stream {
    /// Creates a new stream prepared for decompression.
    ///
    /// If `small` is true, then the library will use an alternative
    /// decompression algorithm which uses less memory but at the cost of
    /// decompressing more slowly (roughly speaking, half the speed, but the
    /// maximum memory requirement drops to around 2300k). See
    pub fn new_decompress(small: bool) -> Stream {
        unsafe {
            let mut raw = Box::new(mem::zeroed());
            assert_eq!(ffi::BZ2_bzDecompressInit(&mut *raw, 0, small as c_int), 0);
            Stream { raw: raw, kind: Kind::Decompress }
        }
    }

    /// Creates a new stream prepared for compression.
    ///
    /// The `work_factor` parameter controls how the compression phase behaves
    /// when presented with worst case, highly repetitive, input data. If
    /// compression runs into difficulties caused by repetitive data, the
    /// library switches from the standard sorting algorithm to a fallback
    /// algorithm. The fallback is slower than the standard algorithm by perhaps
    /// a factor of three, but always behaves reasonably, no matter how bad the
    /// input.
    ///
    /// Lower values of `work_factor` reduce the amount of effort the standard
    /// algorithm will expend before resorting to the fallback. You should set
    /// this parameter carefully; too low, and many inputs will be handled by
    /// the fallback algorithm and so compress rather slowly, too high, and your
    /// average-to-worst case compression times can become very large. The
    /// default value of 30 gives reasonable behaviour over a wide range of
    /// circumstances.
    ///
    /// Allowable values range from 0 to 250 inclusive. 0 is a special case,
    /// equivalent to using the default value of 30.
    pub fn new_compress(lvl: ::Compress, work_factor: u32) -> Stream {
        unsafe {
            let mut raw = Box::new(mem::zeroed());
            assert_eq!(ffi::BZ2_bzCompressInit(&mut *raw, lvl as c_int, 0,
                                               work_factor as c_int), 0);
            Stream { raw: raw, kind: Kind::Compress }
        }
    }

    /// Decompress a block of input into a block of output.
    pub fn decompress(&mut self, input: &[u8], output: &mut [u8]) -> c_int {
        self.raw.next_in = input.as_ptr() as *mut _;
        self.raw.avail_in = input.len() as c_uint;
        self.raw.next_out = output.as_mut_ptr() as *mut _;
        self.raw.avail_out = output.len() as c_uint;
        unsafe { ffi::BZ2_bzDecompress(&mut *self.raw) }
    }

    /// Decompress a block of input into an output vector.
    ///
    /// This function will not grow `output`, but it will fill the space after
    /// its current length up to its capacity. The length of the vector will be
    /// adjusted appropriately.
    pub fn decompress_vec(&mut self, input: &[u8], output: &mut Vec<u8>)
                          -> c_int {
        let cap = output.capacity();
        let len = output.len();
        self.raw.avail_in = input.len() as c_uint;
        self.raw.next_in = input.as_ptr() as *mut _;
        self.raw.avail_out = (cap - len) as c_uint;
        self.raw.next_out = unsafe {
            output.as_mut_ptr().offset(len as isize) as *mut _
        };

        let before = self.total_out();
        let rc = unsafe { ffi::BZ2_bzDecompress(&mut *self.raw) };
        let diff = (self.total_out() - before) as usize;
        unsafe { output.set_len(len + diff) }
        return rc;
    }

    /// Compress a block of input into a block of output.
    ///
    /// If anything other than BZ_OK is seen, `Err` is returned. The action
    /// given must be one of Run, Flush or Finish.
    pub fn compress(&mut self, input: &[u8], output: &mut [u8],
                    action: Action) -> c_int {
        self.raw.next_in = input.as_ptr() as *mut _;
        self.raw.avail_in = input.len() as c_uint;
        self.raw.next_out = output.as_mut_ptr() as *mut _;
        self.raw.avail_out = output.len() as c_uint;
        unsafe { ffi::BZ2_bzCompress(&mut *self.raw, action as c_int) }
    }

    /// Compress a block of input into an output vector.
    ///
    /// This function will not grow `output`, but it will fill the space after
    /// its current length up to its capacity. The length of the vector will be
    /// adjusted appropriately.
    pub fn compress_vec(&mut self, input: &[u8], output: &mut Vec<u8>,
                        action: Action) -> c_int {
        let cap = output.capacity();
        let len = output.len();
        self.raw.avail_in = input.len() as c_uint;
        self.raw.next_in = input.as_ptr() as *mut _;
        self.raw.avail_out = (cap - len) as c_uint;
        self.raw.next_out = unsafe {
            output.as_mut_ptr().offset(len as isize) as *mut _
        };

        let before = self.total_out();
        let rc = unsafe { ffi::BZ2_bzCompress(&mut *self.raw, action as c_int) };
        let diff = (self.total_out() - before) as usize;
        unsafe { output.set_len(len + diff) }
        return rc;
    }

    /// Total number of bytes processed as input
    pub fn total_in(&self) -> u64 {
        (self.raw.total_in_lo32 as u64) |
        ((self.raw.total_in_hi32 as u64) << 32)
    }

    /// Total number of bytes processed as output
    pub fn total_out(&self) -> u64 {
        (self.raw.total_out_lo32 as u64) |
        ((self.raw.total_out_hi32 as u64) << 32)
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(match self.kind {
                Kind::Compress => ffi::BZ2_bzCompressEnd(&mut *self.raw),
                Kind::Decompress => ffi::BZ2_bzDecompressEnd(&mut *self.raw),
            }, 0);
        }
    }
}
