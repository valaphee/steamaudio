use crate::error::check;
use crate::ffi;
use crate::prelude::*;
use glam::Vec3;

pub struct BinauralEffect {
    pub(crate) inner: ffi::IPLBinauralEffect,

    hrtf: Hrtf,
}

impl BinauralEffect {
    pub fn new(
        context: Context,
        sample_rate: u32,
        frame_length: u32,
        hrtf: Hrtf,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLBinauralEffectSettings { hrtf: hrtf.inner };
        let mut effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplBinauralEffectCreate(
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

    pub fn apply(
        &self,
        direction: Vec3,
        interpolation: HrtfInterpolation,
        spatial_bend: f32,
        in_: &Buffer,
        out: &mut Buffer,
    ) {
        let params = ffi::IPLBinauralEffectParams {
            direction: direction.into(),
            interpolation: interpolation.into(),
            spatialBlend: spatial_bend,
            hrtf: self.hrtf.inner,
            peakDelays: std::ptr::null_mut(),
        };

        unsafe {
            ffi::iplBinauralEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    pub fn reset(&self) {
        unsafe {
            ffi::iplBinauralEffectReset(self.inner);
        }
    }
}

unsafe impl Sync for BinauralEffect {}
unsafe impl Send for BinauralEffect {}

impl Clone for BinauralEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplBinauralEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            hrtf: self.hrtf.clone(),
        }
    }
}

impl Drop for BinauralEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplBinauralEffectRelease(&mut self.inner);
        }
    }
}

pub enum HrtfInterpolation {
    Nearest,
    Bilinear,
}

impl From<HrtfInterpolation> for ffi::IPLHRTFInterpolation {
    fn from(value: HrtfInterpolation) -> ffi::IPLHRTFInterpolation {
        match value {
            HrtfInterpolation::Nearest => ffi::IPLHRTFInterpolation_IPL_HRTFINTERPOLATION_NEAREST,
            HrtfInterpolation::Bilinear => ffi::IPLHRTFInterpolation_IPL_HRTFINTERPOLATION_BILINEAR,
        }
    }
}
