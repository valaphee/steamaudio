use crate::{context::Context, error::check, ffi};

impl Context {
    /// Creates an HRTF.
    ///
    /// Calling this function is somewhat expensive; avoid creating HRTF objects
    /// in your audio thread at all if possible.
    ///
    /// This function is not thread-safe. Do not simultaneously call it from
    /// multiple threads.
    pub fn create_hrtf(&self, sampling_rate: u32, frame_size: u32) -> crate::error::Result<Hrtf> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut hrtf_settings = ffi::IPLHRTFSettings {
            type_: ffi::IPLHRTFType_IPL_HRTFTYPE_DEFAULT,
            sofaFileName: std::ptr::null_mut(),
            sofaData: std::ptr::null_mut(),
            sofaDataSize: 0,
            volume: 1.0,
            normType: ffi::IPLHRTFNormType_IPL_HRTFNORMTYPE_NONE,
        };
        let mut hrtf = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplHRTFCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut hrtf_settings,
                    &mut hrtf,
                ),
                Hrtf {
                    inner: hrtf,
                    context: self.clone(),
                },
            )
        }
    }
}

/// A Head-Related Transfer Function (HRTF). HRTFs describe how sound from
/// different directions is perceived by a each of a listener's ears, and are a
/// crucial component of spatial audio. Steam Audio includes a built-in HRTF,
/// while also allowing developers and users to import their own custom HRTFs.
pub struct Hrtf {
    pub(crate) inner: ffi::IPLHRTF,

    context: Context,
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

unsafe impl Sync for Hrtf {}
