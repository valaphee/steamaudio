use crate::error::check;
use crate::ffi;
use std::ffi::CStr;

pub struct Context {
    pub(crate) inner: ffi::IPLContext,
}

impl Context {
    pub fn retain(&self) -> Self {
        unsafe {
            ffi::iplContextRetain(self.inner);
        }

        Self { inner: self.inner }
    }
}

impl Default for Context {
    fn default() -> Self {
        let context_settings = ffi::IPLContextSettings {
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
            check(ffi::iplContextCreate(&context_settings, &mut context), ()).unwrap();
        }

        Self { inner: context }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            ffi::iplContextRelease(&mut self.inner);
        }
    }
}

#[derive(Copy, Clone)]
struct AllocInfo {
    layout: std::alloc::Layout,
    ptr: *mut u8,
}

unsafe extern "C" fn log_callback(level: ffi::IPLLogLevel, message: *const std::os::raw::c_char) {
    match level {
        ffi::IPLLogLevel_IPL_LOGLEVEL_INFO => {
            tracing::info!("{}", CStr::from_ptr(message).to_str().unwrap());
        }
        ffi::IPLLogLevel_IPL_LOGLEVEL_WARNING => {
            tracing::warn!("{}", CStr::from_ptr(message).to_str().unwrap());
        }
        ffi::IPLLogLevel_IPL_LOGLEVEL_ERROR => {
            tracing::error!("{}", CStr::from_ptr(message).to_str().unwrap());
        }
        ffi::IPLLogLevel_IPL_LOGLEVEL_DEBUG => {
            tracing::debug!("{}", CStr::from_ptr(message).to_str().unwrap());
        }
        _ => unreachable!(),
    }
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
            let alloc_info_ptr = ptr.sub(std::mem::size_of::<AllocInfo>()) as *mut AllocInfo;
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
