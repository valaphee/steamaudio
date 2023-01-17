use crate::error::check;
use crate::ffi;
use crate::prelude::*;

pub struct AmbisonicsRotationEffect {
    pub(crate) inner: ffi::IPLAmbisonicsRotationEffect,
}

impl AmbisonicsRotationEffect {
    pub fn new(
        context: Context,
        sample_rate: u32,
        frame_length: u32,
        maximum_order: u8,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLAmbisonicsRotationEffectSettings {
            maxOrder: maximum_order as i32,
        };
        let mut effect: ffi::IPLAmbisonicsRotationEffect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsRotationEffectCreate(
                    context.inner,
                    &audio_settings,
                    &effect_settings,
                    &mut effect,
                ),
                (),
            )?;
        }

        Ok(Self { inner: effect })
    }

    pub fn apply(&self, orientation: Orientation, order: u8, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLAmbisonicsRotationEffectParams {
            orientation: orientation.into(),
            order: order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsRotationEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    pub fn reset(&self) {
        unsafe {
            ffi::iplAmbisonicsRotationEffectReset(self.inner);
        }
    }
}

unsafe impl Sync for AmbisonicsRotationEffect {}
unsafe impl Send for AmbisonicsRotationEffect {}

impl Clone for AmbisonicsRotationEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsRotationEffectRetain(self.inner);
        }

        Self { inner: self.inner }
    }
}

impl Drop for AmbisonicsRotationEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAmbisonicsRotationEffectRelease(&mut self.inner);
        }
    }
}
