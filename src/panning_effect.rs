use crate::error::check;
use crate::ffi;
use crate::prelude::*;
use glam::Vec3;

pub struct PanningEffect {
    pub(crate) inner: ffi::IPLPanningEffect,
}

impl PanningEffect {
    pub fn new(
        context: Context,
        sample_rate: u32,
        frame_length: u32,
        speaker_layout: SpeakerLayout,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLPanningEffectSettings {
            speakerLayout: speaker_layout.into(),
        };
        let mut effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplPanningEffectCreate(
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

    pub fn apply(&self, direction: Vec3, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLPanningEffectParams {
            direction: direction.into(),
        };

        unsafe {
            ffi::iplPanningEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    pub fn reset(&self) {
        unsafe {
            ffi::iplPanningEffectReset(self.inner);
        }
    }
}

unsafe impl Sync for PanningEffect {}
unsafe impl Send for PanningEffect {}

impl Clone for PanningEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplPanningEffectRetain(self.inner);
        }

        Self { inner: self.inner }
    }
}

impl Drop for PanningEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplPanningEffectRelease(&mut self.inner);
        }
    }
}
