use crate::error::check;
use crate::ffi;
use crate::prelude::*;
use glam::Vec3;

pub struct VirtualSurroundEffect {
    pub(crate) inner: ffi::IPLVirtualSurroundEffect,

    hrtf: Hrtf
}

impl VirtualSurroundEffect {
    pub fn new(
        context: Context,
        sample_rate: u32,
        frame_length: u32,
        speaker_layout: SpeakerLayout,
        hrtf: Hrtf,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLVirtualSurroundEffectSettings {
            speakerLayout: speaker_layout.into(),
            hrtf: hrtf.inner
        };
        let mut effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplVirtualSurroundEffectCreate(
                    context.inner,
                    &audio_settings,
                    &effect_settings,
                    &mut effect,
                ),
                (),
            )?;
        }

        Ok(Self { inner: effect, hrtf })
    }

    pub fn apply(&self, direction: Vec3, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLVirtualSurroundEffectParams {
            hrtf: self.hrtf.inner
        };

        unsafe {
            ffi::iplVirtualSurroundEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    pub fn reset(&self) {
        unsafe {
            ffi::iplVirtualSurroundEffectReset(self.inner);
        }
    }
}

unsafe impl Sync for VirtualSurroundEffect {}
unsafe impl Send for VirtualSurroundEffect {}

impl Clone for VirtualSurroundEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplVirtualSurroundEffectRetain(self.inner);
        }

        Self { inner: self.inner, hrtf: self.hrtf.clone() }
    }
}

impl Drop for VirtualSurroundEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplVirtualSurroundEffectRelease(&mut self.inner);
        }
    }
}
