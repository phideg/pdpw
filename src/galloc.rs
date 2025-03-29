use std::alloc::Layout;

pub(crate) struct SecureGlobalAlloc;

unsafe impl std::alloc::GlobalAlloc for SecureGlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { std::alloc::System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            let raw_slice = std::slice::from_raw_parts_mut(ptr, layout.size());
            raw_slice.fill(0xFF);
            std::alloc::System.dealloc(ptr, layout)
        }
    }
}
