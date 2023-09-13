extern crate cc;
extern crate pkg_config;

use std::path::PathBuf;
use std::{env, fmt, fs};

fn main() {
    let mut cfg = cc::Build::new();
    let target = env::var("TARGET").unwrap();
    cfg.warnings(false);

    if target.contains("windows") {
        cfg.define("_WIN32", None);
        cfg.define("BZ_EXPORT", None);
    } else if !cfg!(feature = "static") {
        // pkg-config doesn't guarantee static link
        if pkg_config::Config::new()
            .cargo_metadata(true)
            .probe("bzip2")
            .is_ok()
        {
            return;
        }
    }

    // List out the WASM targets that need wasm-shim.
    // Note that Emscripten already provides its own C standard library so
    // wasm32-unknown-emscripten should not be included here.
    // See: https://github.com/gyscos/zstd-rs/pull/209
    let need_wasm_shim =
        env::var("TARGET").map_or(false, |target| target == "wasm32-unknown-unknown");

    if need_wasm_shim {
        cargo_print(&"rerun-if-changed=wasm_shim/stdlib.h");
        cargo_print(&"rerun-if-changed=wasm_shim/string.h");

        cfg.include("wasm_shim/");
        cfg.opt_level(3);
    }
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_arch == "wasm32" || target_os == "hermit" {
        cargo_print(&"rustc-cfg=feature=\"std\"");
    }

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    cfg.include("bzip2-1.0.8")
        .define("_FILE_OFFSET_BITS", Some("64"))
        .define("BZ_NO_STDIO", None)
        .file("bzip2-1.0.8/blocksort.c")
        .file("bzip2-1.0.8/huffman.c")
        .file("bzip2-1.0.8/crctable.c")
        .file("bzip2-1.0.8/randtable.c")
        .file("bzip2-1.0.8/compress.c")
        .file("bzip2-1.0.8/decompress.c")
        .file("bzip2-1.0.8/bzlib.c")
        .out_dir(dst.join("lib"))
        .compile("libbz2.a");

    let src = env::current_dir().unwrap().join("bzip2-1.0.8");
    let include = dst.join("include");
    fs::create_dir_all(&include).unwrap();
    fs::copy(src.join("bzlib.h"), dst.join("include/bzlib.h")).unwrap();
    cargo_print(&format_args!("cargo:root={}", dst.display()));
    cargo_print(&format_args!(
        "cargo:include={}",
        dst.join("include").display()
    ));
}

/// Print a line for cargo.
///
/// If non-cargo is set, do not print anything.
fn cargo_print(content: &dyn fmt::Display) {
    if cfg!(not(feature = "non-cargo")) {
        println!("cargo:{}", content);
    }
}
