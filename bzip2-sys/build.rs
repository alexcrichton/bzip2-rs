extern crate cc;
extern crate pkg_config;

use std::env;
use std::fs::{self, File};
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let mut cfg = cc::Build::new();
    let target = env::var("TARGET").unwrap();
    cfg.warnings(false);

    if target.contains("windows") {
        cfg.define("_WIN32", None);
        cfg.define("BZ_EXPORT", None);
    } else {
        if pkg_config::Config::new()
            .cargo_metadata(true)
            .probe("bzip2")
            .is_ok()
        {
            return Ok(());
        }
    }

    let dst = PathBuf::from(env::var_os("OUT_DIR").expect("no OUT_DIR env variable"));

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

    let src = env::current_dir()?.join("bzip2-1.0.8");
    let include = dst.join("include");
    fs::create_dir_all(&include)?;
    // note: not using fs::copy because destination file permissions don't matter
    io::copy(
        &mut File::open(src.join("bzlib.h"))?,
        &mut File::create(dst.join("include/bzlib.h"))?,
    )?;
    println!("cargo:root={}", dst.display());
    println!("cargo:include={}", dst.join("include").display());

    Ok(())
}
