# bzip2

[Documentation](https://docs.rs/bzip2)

A streaming compression/decompression library for rust with bindings to `libbz2`.

## Features

By default, `bzip2-rs` attempts to use the system `libbz2`. When `libbz2` is not available, the library 
is built from source. A from source build requires a functional C toolchain for your target, and may not 
work for all targets (in particular webassembly).

*`static`*

Always build `libbz2` from source, and statically link it. 

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this repository by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
