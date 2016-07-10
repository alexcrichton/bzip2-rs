//! Bzip compression for Rust
//!
//! This library contains bindings to libbz2 to support bzip compression and
//! decompression for Rust. The streams offered in this library are primarily
//! found in the `reader` and `writer` modules. Both compressors and
//! decompressors are available in each module depending on what operation you
//! need.
//!
//! Access to the raw decompression/compression stream is also provided through
//! the `raw` module which has a much closer interface to libbz2.
//!
//! # Example
//!
//! ```
//! use std::io::prelude::*;
//! use bzip2::Compression;
//! use bzip2::read::{BzEncoder, BzDecoder};
//!
//! // Round trip some bytes from a byte source, into a compressor, into a
//! // decompressor, and finally into a vector.
//! let data = "Hello, World!".as_bytes();
//! let compressor = BzEncoder::new(data, Compression::Best);
//! let mut decompressor = BzDecoder::new(compressor);
//!
//! let mut contents = String::new();
//! decompressor.read_to_string(&mut contents).unwrap();
//! assert_eq!(contents, "Hello, World!");
//! ```

#![deny(missing_docs, warnings)]
#![doc(html_root_url = "http://alexcrichton.com/bzip2-rs")]

extern crate bzip2_sys as ffi;
extern crate libc;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
extern crate quickcheck;

pub use mem::{Compress, Decompress, Action, Status, Error};

mod mem;

pub mod bufread;
pub mod read;
pub mod write;

/// When compressing data, the compression level can be specified by a value in
/// this enum.
#[derive(Copy, Clone)]
pub enum Compression {
    /// Optimize for the best speed of encoding.
    Fastest = 1,
    /// Optimize for the size of data being encoded.
    Best = 9,
    /// Choose the default compression, a balance between speed and size.
    Default = 6,
}

