use crate::error::check;
use crate::ffi;
use crate::prelude::*;

pub struct AmbisonicsPanningEffect {
    pub(crate) inner: ffi::IPLAmbisonicsPanningEffect,
}

impl AmbisonicsPanningEffect {
    pub fn new(
        context: Context,
        sample_rate: u32,
        frame_length: u32,
        speaker_layout: SpeakerLayout,
        maximum_order: u8,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLAmbisonicsPanningEffectSettings {
            speakerLayout: speaker_layout.into(),
            maxOrder: maximum_order as i32,
        };
        let mut effect: ffi::IPLAmbisonicsPanningEffect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsPanningEffectCreate(
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

    pub fn apply(&self, order: u8, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLAmbisonicsPanningEffectParams {
            order: order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsPanningEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    pub fn reset(&self) {
        unsafe {
            ffi::iplAmbisonicsPanningEffectReset(self.inner);
        }
    }
}

unsafe impl Sync for AmbisonicsPanningEffect {}
unsafe impl Send for AmbisonicsPanningEffect {}

impl Clone for AmbisonicsPanningEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsPanningEffectRetain(self.inner);
        }

        Self { inner: self.inner }
    }
}

impl Drop for AmbisonicsPanningEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAmbisonicsPanningEffectRelease(&mut self.inner);
        }
    }
}
