# bzip2

[Documentation](https://docs.rs/bzip2)

A streaming compression/decompression library for rust with bindings to libbz2.

```toml
# Cargo.toml
[dependencies]
bzip2 = "0.4"
```

## WASM
bzip2-rs can be compiled to WASM. Make sure you added `wasm32-unknown-unknown` target
```bash
rustup target add wasm32-unknown-unknown
```
To build and run WASM example make sure that you working directory in terminal is `bzip2-sys`
### Build WASM target
```bash
cargo build --target wasm32-unknown-unknown --no-default-features --example it_work
```

### Run WASM target using wasmtime
```bash
wasmtime ..\target\wasm32-unknown-unknown\debug\examples\it_work.wasm --invoke test_decompress
```

# License

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
