mod platform;

use std::{alloc::Layout, ptr::NonNull};

pub(crate) type Pointer<T> = Option<NonNull<T>>;

pub(crate) struct SecureGlobalAlloc;

unsafe impl std::alloc::GlobalAlloc for SecureGlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.allocate(layout) {
            Ok(address) => address.cast().as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        platform::return_memory(NonNull::new_unchecked(ptr));
    }
}
