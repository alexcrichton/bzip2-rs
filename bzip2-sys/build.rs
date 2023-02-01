extern crate cc;
extern crate pkg_config;

use std::path::PathBuf;
use std::{env, fs};

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

    let dst = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    cfg.flag_if_supported("-ffunction-sections")
        .flag_if_supported("-fdata-sections")
        .flag_if_supported("-fmerge-all-constants");

    if cfg!(feature = "fat-lto") {
        cfg.flag_if_supported("-flto");
    } else if cfg!(feature = "thin-lto") {
        if cfg.is_flag_supported("-flto=thin").unwrap_or(false) {
            cfg.flag("-flto=thin");
        } else {
            cfg.flag_if_supported("-flto");
        }
    }

    if cfg!(feature = "thin") {
        cfg.opt_level_str("z");
    }

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
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}", dst.join("include").display());
}
