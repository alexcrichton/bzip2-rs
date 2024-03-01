# WASM shims

This directory contains some WASM shims for C-functions used by bzip2 that are not available otherwise for the `wasm32-unknown-unknow` target.
Specifically, these are:

- `malloc`
- `calloc`
- `free`
- `memset`
- `memcpy`
- `memmove`

The shims are implemented in Rust and exposed as C functions that the bzip2-sys crate can then use / link against.
