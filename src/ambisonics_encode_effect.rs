use crate::error::check;
use crate::ffi;
use crate::prelude::*;
use glam::Vec3;

pub struct AmbisonicsEncodeEffect {
    pub(crate) inner: ffi::IPLAmbisonicsEncodeEffect,
}

impl AmbisonicsEncodeEffect {
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
        let effect_settings = ffi::IPLAmbisonicsEncodeEffectSettings {
            maxOrder: maximum_order as i32,
        };
        let mut effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsEncodeEffectCreate(
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

    pub fn apply(&self, direction: Vec3, order: u8, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLAmbisonicsEncodeEffectParams {
            direction: direction.into(),
            order: order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsEncodeEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    pub fn reset(&self) {
        unsafe {
            ffi::iplAmbisonicsEncodeEffectReset(self.inner);
        }
    }
}

unsafe impl Sync for AmbisonicsEncodeEffect {}
unsafe impl Send for AmbisonicsEncodeEffect {}

impl Clone for AmbisonicsEncodeEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsEncodeEffectRetain(self.inner);
        }

        Self { inner: self.inner }
    }
}

impl Drop for AmbisonicsEncodeEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAmbisonicsEncodeEffectRelease(&mut self.inner);
        }
    }
}
