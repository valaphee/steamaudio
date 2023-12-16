use tracing::{debug, error, info, warn};

use crate::{
    error::{check, Result},
    ffi,
};

/// A context object, which controls low-level operations of Steam Audio.
/// Typically, a context is specified once during the execution of the client
/// program, before calling any other API functions
pub struct Context {
    pub(crate) inner: ffi::IPLContext,
}

impl Context {
    /// Creates a context object. A context must be created before creating any
    /// other API objects.
    pub fn new() -> Result<Self> {
        unsafe extern "C" fn log_callback(
            level: ffi::IPLLogLevel,
            message: *const std::os::raw::c_char,
        ) {
            let message = std::ffi::CStr::from_ptr(message).to_str().unwrap();
            match level {
                ffi::IPLLogLevel_IPL_LOGLEVEL_INFO => {
                    info!(message);
                }
                ffi::IPLLogLevel_IPL_LOGLEVEL_WARNING => {
                    warn!(message);
                }
                ffi::IPLLogLevel_IPL_LOGLEVEL_ERROR => {
                    error!(message);
                }
                ffi::IPLLogLevel_IPL_LOGLEVEL_DEBUG => {
                    debug!(message);
                }
                _ => unreachable!(),
            }
        }

        struct AllocInfo {
            layout: std::alloc::Layout,
            ptr: *mut u8,
        }

        unsafe extern "C" fn allocate_callback(
            size: ffi::IPLsize,
            alignment: ffi::IPLsize,
        ) -> *mut std::ffi::c_void {
            std::alloc::Layout::from_size_align(size, alignment).map_or_else(
                |_| std::ptr::null_mut(),
                |layout| {
                    let alloc_info_layout = std::alloc::Layout::new::<AllocInfo>();
                    let (alloc_layout, offset) = alloc_info_layout.extend(layout).unwrap();

                    let alloc_ptr = std::alloc::alloc(alloc_layout);
                    if alloc_ptr.is_null() {
                        return alloc_ptr;
                    }

                    let ptr = alloc_ptr.add(offset);
                    let alloc_info_ptr =
                        ptr.sub(std::mem::size_of::<AllocInfo>()) as *mut AllocInfo;
                    alloc_info_ptr.write_unaligned(AllocInfo {
                        layout: alloc_layout,
                        ptr: alloc_ptr,
                    });

                    ptr
                },
            ) as *mut std::ffi::c_void
        }

        unsafe extern "C" fn free_callback(ptr: *mut std::ffi::c_void) {
            assert!(!ptr.is_null());

            let alloc_info_ptr = ptr.sub(std::mem::size_of::<AllocInfo>()) as *const AllocInfo;
            let alloc_info = alloc_info_ptr.read_unaligned();
            std::alloc::dealloc(alloc_info.ptr, alloc_info.layout);
        }

        let mut context_settings = ffi::IPLContextSettings {
            version: ffi::STEAMAUDIO_VERSION_MAJOR << 16
                | ffi::STEAMAUDIO_VERSION_MINOR << 8
                | ffi::STEAMAUDIO_VERSION_PATCH,
            logCallback: Some(log_callback),
            allocateCallback: Some(allocate_callback),
            freeCallback: Some(free_callback),
            simdLevel: ffi::IPLSIMDLevel_IPL_SIMDLEVEL_AVX512,
        };
        let mut context = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplContextCreate(&mut context_settings, &mut context),
                Self { inner: context },
            )
        }
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplContextRetain(self.inner);
        }

        Self { inner: self.inner }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            ffi::iplContextRelease(&mut self.inner);
        }
    }
}

unsafe impl Send for Context {}

unsafe impl Sync for Context {}
