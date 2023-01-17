use crate::error::check;
use crate::ffi;
use crate::prelude::*;

pub struct DirectEffect {
    pub(crate) inner: ffi::IPLDirectEffect,
}

impl DirectEffect {
    pub fn new(
        context: Context,
        sample_rate: u32,
        frame_length: u32,
        channels: u16,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLDirectEffectSettings {
            numChannels: channels as i32,
        };
        let mut effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplDirectEffectCreate(
                    context.inner,
                    &audio_settings,
                    &effect_settings,
                    &mut effect,
                ),
                (),
            )?;
        }

        Ok(DirectEffect { inner: effect })
    }

    pub fn apply(
        &self,
        distance_attenuation: Option<f32>,
        air_absorption: Option<[f32; 3]>,
        directivity: Option<f32>,
        occlusion: Option<f32>,
        transmission: Option<(TransmissionType, [f32; 3])>,
        in_: &Buffer,
        out: &mut Buffer,
    ) {
        let mut params: ffi::IPLDirectEffectParams = unsafe { std::mem::zeroed() };
        if let Some(distance_attenuation) = distance_attenuation {
            params.flags |=
                ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYDISTANCEATTENUATION;
            params.distanceAttenuation = distance_attenuation;
        }
        if let Some(air_absorption) = air_absorption {
            params.flags |= ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYAIRABSORPTION;
            params.airAbsorption = air_absorption;
        }
        if let Some(directivity) = directivity {
            params.flags |= ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYDIRECTIVITY;
            params.directivity = directivity;
        }
        if let Some(occlusion) = occlusion {
            params.flags |= ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYOCCLUSION;
            params.occlusion = occlusion;
        }
        if let Some((transmission_type, transmission)) = transmission {
            params.flags |= ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYTRANSMISSION;
            params.transmissionType = transmission_type.into();
            params.transmission = transmission;
        }

        unsafe {
            ffi::iplDirectEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    pub fn reset(&self) {
        unsafe {
            ffi::iplDirectEffectReset(self.inner);
        }
    }
}

unsafe impl Sync for DirectEffect {}
unsafe impl Send for DirectEffect {}

impl Clone for DirectEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplDirectEffectRetain(self.inner);
        }

        Self { inner: self.inner }
    }
}

impl Drop for DirectEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplDirectEffectRelease(&mut self.inner);
        }
    }
}

pub enum TransmissionType {
    FrequencyIndependent,
    FrequencyDependent,
}

impl From<TransmissionType> for ffi::IPLTransmissionType {
    fn from(value: TransmissionType) -> ffi::IPLHRTFInterpolation {
        match value {
            TransmissionType::FrequencyIndependent => {
                ffi::IPLTransmissionType_IPL_TRANSMISSIONTYPE_FREQINDEPENDENT
            }
            TransmissionType::FrequencyDependent => {
                ffi::IPLTransmissionType_IPL_TRANSMISSIONTYPE_FREQDEPENDENT
            }
        }
    }
}
