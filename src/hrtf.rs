use crate::{context::Context, ffi};

/// A Head-Related Transfer Function (HRTF). HRTFs describe how sound from
/// different directions is perceived by a each of a listener's ears, and are a
/// crucial component of spatial audio. Steam Audio includes a built-in HRTF,
/// while also allowing developers and users to import their own custom HRTFs.
pub struct Hrtf {
    pub(crate) inner: ffi::IPLHRTF,

    pub(crate) context: Context,
}

impl Clone for Hrtf {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplHRTFRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
        }
    }
}

impl Drop for Hrtf {
    fn drop(&mut self) {
        unsafe {
            ffi::iplHRTFRelease(&mut self.inner);
        }
    }
}

unsafe impl Send for Hrtf {}
