use std::ptr::NonNull;

use super::Pointer;

/// Convinience wrapper for [`PlatformSpecificMemory::request_memory`].
#[inline]
pub(crate) unsafe fn request_memory(length: usize) -> Pointer<u8> {
    _platform::request_memory(length)
}

/// Convinience wrapper for [`PlatformSpecificMemory::return_memory`].
#[inline]
pub(crate) unsafe fn return_memory(address: NonNull<u8>, length: usize) {
    _platform::return_memory(address, length)
}

#[cfg(unix)]
mod _platform {
    use super::Pointer;
    use libc;
    use std::ptr::{self, NonNull};

    pub(super) unsafe fn request_memory(length: usize) -> Pointer<u8> {
        // Memory protection. Read-Write only.
        let protection = libc::PROT_READ | libc::PROT_WRITE;
        // Memory should be private to our process and not mapped to any file.
        let flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;
        // For all the configuration options that `mmap` accepts see
        // https://man7.org/linux/man-pages/man2/mmap.2.html
        match libc::mmap(ptr::null_mut(), length, protection, flags, -1, 0) {
            libc::MAP_FAILED => None,
            address => Some(NonNull::new_unchecked(address).cast()),
        }
    }

    pub(super) unsafe fn return_memory(address: NonNull<u8>, length: usize) {
        let addr_ptr = address.cast().as_ptr();
        libc::memset(addr_ptr, 0, length);
        if libc::munmap(addr_ptr, length) != 0 {
            // TODO: What should we do here? Panic? Memory region is still
            // valid here, it wasn't unmapped.
        }
    }

    pub(super) unsafe fn page_size() -> usize {
        libc::sysconf(libc::_SC_PAGE_SIZE) as usize
    }
}

#[cfg(windows)]
mod _plaform {
    use crate::Pointer;
    use std::{mem::MaybeUninit, ptr::NonNull};
    use windows::Win32::System::{Memory, SystemInformation};

    pub(super) unsafe fn request_memory(length: usize) -> Pointer<u8> {
        // Similar to mmap on Linux, Read-Write only.
        let protection = Memory::PAGE_READWRITE;
        // This works a little bit different from mmap, memory has to be
        // reserved first and then committed in order to become usable. We
        // can do both at the same time with one single call.
        let flags = Memory::MEM_RESERVE | Memory::MEM_COMMIT;
        // For more detailed explanations of each parameter, see
        // https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualalloc#parameters
        let address = Memory::VirtualAlloc(None, length, flags, protection);
        NonNull::new(address.cast())
    }

    pub(super) unsafe fn return_memory(address: NonNull<u8>, _length: usize) {
        // Again, we have to decommit memory first and then release it. We
        // can skip decommitting by specifying length of 0 and MEM_RELEASE
        // flag. See the docs for details:
        // https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualfree#parameters
        let address = address.cast().as_ptr();
        let length = 0;
        let flags = Memory::MEM_RELEASE;
        if !Memory::VirtualFree(address, length, flags).as_bool() {
            // TODO: Release failed, don't know what to do here yet. Same
            // problem as munmap on Linux.
        }
    }

    pub(super) unsafe fn page_size() -> usize {
        let mut system_info = MaybeUninit::uninit();
        SystemInformation::GetSystemInfo(system_info.as_mut_ptr());
        system_info.assume_init().dwPageSize as usize
    }
}
