use std::alloc::{alloc, alloc_zeroed, dealloc, Layout};

const ALIGNMENT: usize = 16;

#[no_mangle]
pub extern "C" fn rust_bzip2_wasm_shim_malloc(size: usize) -> *mut u8 {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, ALIGNMENT);
        alloc(layout)
    }
}

#[no_mangle]
pub extern "C" fn rust_bzip2_wasm_shim_calloc(nmemb: usize, size: usize) -> *mut u8 {
    let total_size = nmemb * size;
    unsafe {
        let layout = Layout::from_size_align_unchecked(total_size, ALIGNMENT);
        alloc_zeroed(layout)
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_bzip2_wasm_shim_free(ptr: *mut u8) {
    // layout is not actually used
    unsafe {
        let layout = Layout::from_size_align_unchecked(1, ALIGNMENT);
        dealloc(ptr.cast(), layout);
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_bzip2_wasm_shim_memcpy(
    dest: *mut u8,
    src: *const u8,
    n: usize,
) -> *mut u8 {
    std::ptr::copy_nonoverlapping(src, dest, n);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn rust_bzip2_wasm_shim_memmove(
    dest: *mut u8,
    src: *const u8,
    n: usize,
) -> *mut u8 {
    std::ptr::copy(src, dest, n);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn rust_bzip2_wasm_shim_memset(dest: *mut u8, c: i32, n: usize) -> *mut u8 {
    std::ptr::write_bytes(dest, c as u8, n);
    dest
}
