extern crate libc;

use libc::{c_int, c_uint, c_void, c_char};

pub static BZ_RUN: c_int = 0;
pub static BZ_FLUSH: c_int = 1;
pub static BZ_FINISH: c_int = 2;

pub static BZ_OK: c_int = 0;
pub static BZ_RUN_OK: c_int = 1;
pub static BZ_FLUSH_OK: c_int = 2;
pub static BZ_FINISH_OK: c_int = 3;
pub static BZ_STREAM_END: c_int = 4;
pub static BZ_SEQUENCE_ERROR: c_int = -1;
pub static BZ_PARAM_ERROR: c_int = -2;
pub static BZ_MEM_ERROR: c_int = -3;
pub static BZ_DATA_ERROR: c_int = -4;
pub static BZ_DATA_ERROR_MAGIC: c_int = -5;
pub static BZ_IO_ERROR: c_int = -6;
pub static BZ_UNEXPECTED_EOF: c_int = -7;
pub static BZ_OUTBUFF_FULL: c_int = -8;
pub static BZ_CONFIG_ERROR: c_int = -9;

#[repr(C)]
pub struct bz_stream {
    pub next_in: *mut c_char,
    pub avail_in: c_uint,
    pub total_in_lo32: c_uint,
    pub total_in_hi32: c_uint,

    pub next_out: *mut c_char,
    pub avail_out: c_uint,
    pub total_out_lo32: c_uint,
    pub total_out_hi32: c_uint,

    pub state: *mut c_void,

    pub bzalloc: Option<extern fn(*mut c_void, c_int, c_int) -> *mut c_void>,
    pub bzfree: Option<extern fn(*mut c_void, *mut c_void)>,
    pub opaque: *mut c_void,
}

#[link(name = "bz2", kind = "static")]
extern {
    pub fn BZ2_bzCompressInit(stream: *mut bz_stream,
                              blockSize100k: c_int,
                              verbosity: c_int,
                              workFactor: c_int) -> c_int;
    pub fn BZ2_bzCompress(stream: *mut bz_stream, action: c_int) -> c_int;
    pub fn BZ2_bzCompressEnd(stream: *mut bz_stream) -> c_int;
    pub fn BZ2_bzDecompressInit(stream: *mut bz_stream,
                                verbosity: c_int,
                                small: c_int) -> c_int;
    pub fn BZ2_bzDecompress(stream: *mut bz_stream) -> c_int;
    pub fn BZ2_bzDecompressEnd(stream: *mut bz_stream) -> c_int;
}

#[no_mangle]
pub fn bz_internal_error(errcode: c_int) {
    fail!("bz internal error: {}", errcode);
}
