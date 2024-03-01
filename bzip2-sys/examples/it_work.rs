use std::os::raw::{c_char, c_int, c_uint};

#[no_mangle]
pub extern "C" fn test_decompress() -> bool {
    let uncompressed_bytes = include_bytes!("../bzip2-1.0.8/sample1.ref");
    let compressed_bytes = include_bytes!("../bzip2-1.0.8/sample1.bz2");
    let mut raw: Box<bzip2_sys::bz_stream> = unsafe { Box::new(std::mem::zeroed()) };
    let mut buf: [u8; 100352] = [0; 98 * 1024];
    unsafe {
        assert_eq!(bzip2_sys::BZ2_bzDecompressInit(&mut *raw, 0, 0 as c_int), 0);
        raw.next_in = compressed_bytes.as_ptr() as *mut c_char;
        raw.avail_in = compressed_bytes.len().min(c_uint::MAX as usize) as c_uint;

        raw.next_out = buf.as_mut_ptr() as *mut c_char;
        raw.avail_out = buf.len() as c_uint;
        assert_eq!(
            bzip2_sys::BZ2_bzDecompress(&mut *raw),
            bzip2_sys::BZ_STREAM_END
        );
        bzip2_sys::BZ2_bzDecompressEnd(&mut *raw);
    };
    let total_out = ((raw.total_out_lo32 as u64) | ((raw.total_out_hi32 as u64) << 32)) as usize;
    assert_eq!(total_out, uncompressed_bytes.len());

    let slice: &[u8] = buf[0..total_out].as_ref();
    assert_eq!(uncompressed_bytes, slice);

    return true;
}

fn main() {}
