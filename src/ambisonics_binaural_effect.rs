use crate::error::check;
use crate::ffi;
use crate::prelude::*;

pub struct AmbisonicsBinauralEffect {
    pub(crate) inner: ffi::IPLAmbisonicsBinauralEffect,

    hrtf: Hrtf,
}

impl AmbisonicsBinauralEffect {
    pub fn new(
        context: Context,
        sample_rate: u32,
        frame_length: u32,
        hrtf: Hrtf,
        maximum_order: u8,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLAmbisonicsBinauralEffectSettings {
            hrtf: hrtf.inner,
            maxOrder: maximum_order as i32,
        };
        let mut effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsBinauralEffectCreate(
                    context.inner,
                    &audio_settings,
                    &effect_settings,
                    &mut effect,
                ),
                (),
            )?;
        }

        Ok(Self {
            inner: effect,
            hrtf,
        })
    }

    pub fn apply(&self, order: u8, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLAmbisonicsBinauralEffectParams {
            hrtf: self.hrtf.inner,
            order: order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsBinauralEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    pub fn reset(&self) {
        unsafe {
            ffi::iplAmbisonicsBinauralEffectReset(self.inner);
        }
    }
}

unsafe impl Sync for AmbisonicsBinauralEffect {}
unsafe impl Send for AmbisonicsBinauralEffect {}

impl Clone for AmbisonicsBinauralEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsBinauralEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            hrtf: self.hrtf.clone(),
        }
    }
}

impl Drop for AmbisonicsBinauralEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAmbisonicsBinauralEffectRelease(&mut self.inner);
        }
    }
}
