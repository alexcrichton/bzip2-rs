extern crate gcc;

use std::default::Default;

fn main() {
    gcc::compile_library("libbz2.a", &gcc::Config {
        include_directories: vec![Path::new("bzip2-1.0.6")],
        definitions: vec![("BZ_NO_STDIO".to_string(), None)],
        .. Default::default()
    }, &[
        "bzip2-1.0.6/blocksort.c",
        "bzip2-1.0.6/huffman.c",
        "bzip2-1.0.6/crctable.c",
        "bzip2-1.0.6/randtable.c",
        "bzip2-1.0.6/compress.c",
        "bzip2-1.0.6/decompress.c",
        "bzip2-1.0.6/bzlib.c",
    ]);
}
