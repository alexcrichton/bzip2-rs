use std::alloc::{alloc, alloc_zeroed, dealloc, Layout};
use std::mem;
use std::ptr;

// Define a struct to hold the size before the allocated memory
#[repr(C)]
struct AllocationHeader {
    size: usize,
}

const HEADER_SIZE: usize = mem::size_of::<AllocationHeader>();
const ALIGNMENT: usize = 2 * mem::size_of::<usize>();

// Helper function to create a layout that includes the header
fn layout_with_header(size: usize) -> Layout {
    let adjusted_size = HEADER_SIZE + size;
    Layout::from_size_align(adjusted_size, ALIGNMENT).expect("Layout creation failed")
}

#[no_mangle]
pub extern "C" fn rust_bzip2_wasm_shim_malloc(size: usize) -> *mut u8 {
    unsafe {
        let layout = layout_with_header(size);
        let ptr = alloc(layout) as *mut AllocationHeader;
        if !ptr.is_null() {
            // Store the original size in the header
            (*ptr).size = size;
            // Return a pointer to the memory after the header
            ptr.add(1) as *mut u8 
        } else {
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_bzip2_wasm_shim_calloc(nmemb: usize, size: usize) -> *mut u8 {
    let total_size = nmemb * size;
    unsafe {
        let layout = layout_with_header(total_size);
        let ptr = alloc_zeroed(layout) as *mut AllocationHeader;
        if !ptr.is_null() {
            (*ptr).size = total_size;
            ptr.add(1) as *mut u8
        } else {
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_bzip2_wasm_shim_free(ptr: *mut u8) {
    if !ptr.is_null() {
        let ptr = (ptr as *mut AllocationHeader).sub(1); // Move back to the header
        let size = (*ptr).size;
        let layout = layout_with_header(size);
        dealloc(ptr as *mut u8, layout);
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