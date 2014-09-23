//! dox

#![feature(unsafe_destructor)]
#![deny(missing_doc)]

extern crate "libbz2-sys" as ffi;
extern crate libc;

use std::io::MemWriter;

pub mod raw;
pub mod writer;
pub mod reader;

/// Compress a block of input data into a bzip2 encoded output vector.
pub fn compress(data: &[u8], level: CompressionLevel) -> Vec<u8> {
    let mut wr = writer::BzCompressor::new(MemWriter::new(), level);
    wr.write(data).unwrap();
    wr.unwrap().unwrap().unwrap()
}

/// Decompress a block of compressed input data into a raw output vector.
pub fn decompress(data: &[u8]) -> Vec<u8> {
    let mut wr = writer::BzDecompressor::new(MemWriter::new());
    wr.write(data).unwrap();
    wr.unwrap().unwrap().unwrap()
}

/// When compressing data, the compression level can be specified by a value in
/// this enum.
pub enum CompressionLevel {
    /// Optimize for the best speed of encoding.
    BestSpeed = 1,
    /// Optimize for the size of data being encoded.
    BestCompression = 9,
    /// Choose the default compression, a balance between speed and size.
    Default = 6,
}

