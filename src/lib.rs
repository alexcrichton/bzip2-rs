//! Bzip compression for Rust
//!
//! This library contains bindings to libbz2 to support bzip compression and
//! decompression for Rust. The streams offered in this library are primarily
//! found in the `reader` and `writer` modules. Both compressors and
//! decompressors are available in each module depending on what operation you
//! need.
//!
//! Access to the raw decompression/compression stream is also provided through
//! the `raw` module which has a much closer interfact to libbz2.
//!
//! # Example
//!
//! ```
//! use std::io::BufReader;
//! use bzip2::reader::{BzCompressor, BzDecompressor};
//!
//! // Round trip some bytes from a byte source, into a compressor, into a
//! // decompressor, and finally into a vector.
//! let data = BufReader::new(b"Hello, World!");
//! let compressor = BzCompressor::new(data, bzip2::BestCompression);
//! let mut decompressor = BzDecompressor::new(compressor);
//!
//! let contents = decompressor.read_to_string().unwrap();
//! assert_eq!(contents.as_slice(), "Hello, World!");
//! ```

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

