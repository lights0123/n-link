use std::alloc;
use std::alloc::Layout;
use std::os::raw::c_void;

const ALIGN: usize = 8;

#[no_mangle]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    let new_size = size + 8;
    let ptr = alloc::alloc(Layout::from_size_align_unchecked(new_size, ALIGN)) as *mut c_void;
    (ptr as *mut usize).write(new_size);
    ptr.add(8)
}

#[no_mangle]
pub unsafe extern "C" fn free(p: *mut c_void) {
    let ptr = p.sub(8);
    let size = (ptr as *mut usize).read();
    alloc::dealloc(
        ptr as *mut u8,
        Layout::from_size_align_unchecked(size, ALIGN),
    )
}

#[no_mangle]
pub unsafe extern "C" fn realloc(p: *mut c_void, new_size: usize) -> *mut c_void {
    if p.is_null() {
        malloc(new_size)
    } else {
        let ptr = p.sub(8);
        let old_size = (ptr as *mut usize).read();
        let new_ptr = alloc::realloc(
            ptr as *mut u8,
            Layout::from_size_align_unchecked(old_size, ALIGN),
            new_size + 8,
        );
        if new_ptr.is_null() {
            new_ptr as *mut c_void
        } else {
            (ptr as *mut usize).write(new_size + 8);
            new_ptr.add(8) as *mut c_void
        }
    }
}
