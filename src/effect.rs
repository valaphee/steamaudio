use glam::Vec3;

use crate::error::check;
use crate::ffi;
use crate::prelude::*;

/// Pans a single-channel point source to a multi-channel speaker layout based on the 3D position of the source relative to the listener.
pub struct PanningEffect {
    pub(crate) inner: ffi::IPLPanningEffect,

    context: Context,
}

impl PanningEffect {
    /// Creates a panning effect.
    pub fn new(
        context: &Context,
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

        Ok(Self {
            inner: effect,
            context: context.clone(),
        })
    }

    /// Applies a panning effect to an audio buffer.
    ///
    /// This effect CANNOT be applied in-place.
    pub fn apply(&mut self, direction: Vec3, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLPanningEffectParams {
            direction: direction.into(),
        };

        unsafe {
            ffi::iplPanningEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    /// Resets the internal processing state of a panning effect.
    pub fn reset(&mut self) {
        unsafe {
            ffi::iplPanningEffectReset(self.inner);
        }
    }
}

unsafe impl Send for PanningEffect {}

impl Clone for PanningEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplPanningEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
        }
    }
}

impl Drop for PanningEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplPanningEffectRelease(&mut self.inner);
        }
    }
}

/// Spatializes a point source using an HRTF, based on the 3D position of the source relative to the listener.
///
/// The source audio can be 1- or 2-channel; in either case all input channels are spatialized from the same position.
pub struct BinauralEffect {
    pub(crate) inner: ffi::IPLBinauralEffect,

    context: Context,
    hrtf: Hrtf,
}

impl BinauralEffect {
    /// Creates a binaural effect.
    pub fn new(
        context: &Context,
        sample_rate: u32,
        frame_length: u32,
        hrtf: &Hrtf,
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
            context: context.clone(),
            hrtf: hrtf.clone(),
        })
    }

    /// Applies a binaural effect to an audio buffer.
    ///
    /// This effect CANNOT be applied in-place.
    pub fn apply(
        &mut self,
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

    /// Resets the internal processing state of a binaural effect.
    pub fn reset(&mut self) {
        unsafe {
            ffi::iplBinauralEffectReset(self.inner);
        }
    }
}

unsafe impl Send for BinauralEffect {}

impl Clone for BinauralEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplBinauralEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
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

/// Techniques for interpolating HRTF data.
///
/// This is used when rendering a point source whose position relative to the listener is not contained in the measured HRTF data.
pub enum HrtfInterpolation {
    /// Nearest-neighbor filtering, i.e., no interpolation.
    ///
    /// Selects the measurement location that is closest to the source’s actual location.
    Nearest,

    /// Bilinear filtering.
    ///
    /// Incurs a relatively high CPU overhead as compared to nearest-neighbor filtering, so use this for sounds where it has a significant benefit. Typically, bilinear filtering is most useful for wide-band noise-like sounds, such as radio static, mechanical noise, fire, etc.
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

/// Spatializes multi-channel speaker-based audio (e.g., stereo, quadraphonic, 5.1, or 7.1) using HRTF-based binaural rendering.
///
/// The audio signal for each speaker is spatialized from a point in space corresponding to the speaker’s location. This allows users to experience a surround sound mix over regular stereo headphones.
///
/// Virtual surround is also a fast way to get approximate binaural rendering. All sources can be panned to some surround format (say, 7.1). After the sources are mixed, the mix can be rendered using virtual surround. This can reduce CPU usage, at the cost of spatialization accuracy.
pub struct VirtualSurroundEffect {
    pub(crate) inner: ffi::IPLVirtualSurroundEffect,

    context: Context,
    hrtf: Hrtf,
}

impl VirtualSurroundEffect {
    /// Creates a virtual surround effect.
    pub fn new(
        context: &Context,
        sample_rate: u32,
        frame_length: u32,
        speaker_layout: SpeakerLayout,
        hrtf: &Hrtf,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLVirtualSurroundEffectSettings {
            speakerLayout: speaker_layout.into(),
            hrtf: hrtf.inner,
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

        Ok(Self {
            inner: effect,
            context: context.clone(),
            hrtf: hrtf.clone(),
        })
    }

    /// Applies a virtual surround effect to an audio buffer.
    ///
    /// This effect CANNOT be applied in-place.
    pub fn apply(&mut self, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLVirtualSurroundEffectParams {
            hrtf: self.hrtf.inner,
        };

        unsafe {
            ffi::iplVirtualSurroundEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    /// Resets the internal processing state of a virtual surround effect.
    pub fn reset(&mut self) {
        unsafe {
            ffi::iplVirtualSurroundEffectReset(self.inner);
        }
    }
}

unsafe impl Send for VirtualSurroundEffect {}

impl Clone for VirtualSurroundEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplVirtualSurroundEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
            hrtf: self.hrtf.clone(),
        }
    }
}

impl Drop for VirtualSurroundEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplVirtualSurroundEffectRelease(&mut self.inner);
        }
    }
}

/// Encodes a point source into Ambisonics.
///
/// Given a point source with some direction relative to the listener, this effect generates an Ambisonic audio buffer that approximates a point source in the given direction. This allows multiple point sources and ambiences to mixed to a single Ambisonics buffer before being spatialized.
pub struct AmbisonicsEncodeEffect {
    pub(crate) inner: ffi::IPLAmbisonicsEncodeEffect,

    context: Context,
}

impl AmbisonicsEncodeEffect {
    /// Creates an Ambisonics encode effect.
    pub fn new(
        context: &Context,
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

        Ok(Self {
            inner: effect,
            context: context.clone(),
        })
    }

    /// Applies an Ambisonics encode effect to an audio buffer.
    ///
    /// This effect CANNOT be applied in-place.
    pub fn apply(&mut self, direction: Vec3, order: u8, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLAmbisonicsEncodeEffectParams {
            direction: direction.into(),
            order: order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsEncodeEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    /// Resets the internal processing state of an Ambisonics encode effect.
    pub fn reset(&mut self) {
        unsafe {
            ffi::iplAmbisonicsEncodeEffectReset(self.inner);
        }
    }
}

unsafe impl Send for AmbisonicsEncodeEffect {}

impl Clone for AmbisonicsEncodeEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsEncodeEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
        }
    }
}

impl Drop for AmbisonicsEncodeEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAmbisonicsEncodeEffectRelease(&mut self.inner);
        }
    }
}

/// Renders Ambisonic audio by panning it to a standard speaker layout.
///
/// This involves calculating signals to emit from each speaker so as to approximate the Ambisonic sound field.
pub struct AmbisonicsPanningEffect {
    pub(crate) inner: ffi::IPLAmbisonicsPanningEffect,
}

impl AmbisonicsPanningEffect {
    /// Creates an Ambisonics panning effect.
    pub fn new(
        context: &Context,
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

    /// Applies an Ambisonics panning effect to an audio buffer.
    ///
    /// This effect CANNOT be applied in-place.
    pub fn apply(&mut self, order: u8, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLAmbisonicsPanningEffectParams {
            order: order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsPanningEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    /// Resets the internal processing state of an Ambisonics panning effect.
    pub fn reset(&mut self) {
        unsafe {
            ffi::iplAmbisonicsPanningEffectReset(self.inner);
        }
    }
}

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

/// Renders Ambisonic audio using HRTF-based binaural rendering.
///
/// This results in more immersive spatialization of the Ambisonic audio as compared to using an Ambisonics panning effect, at the cost of slightly increased CPU usage.
pub struct AmbisonicsBinauralEffect {
    pub(crate) inner: ffi::IPLAmbisonicsBinauralEffect,

    context: Context,
    hrtf: Hrtf,
}

impl AmbisonicsBinauralEffect {
    /// Creates an Ambisonics binaural effect.
    pub fn new(
        context: &Context,
        sample_rate: u32,
        frame_length: u32,
        hrtf: &Hrtf,
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
            context: context.clone(),
            hrtf: hrtf.clone(),
        })
    }

    /// Applies an Ambisonics binaural effect to an audio buffer.
    ///
    /// This effect CANNOT be applied in-place.
    pub fn apply(&mut self, order: u8, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLAmbisonicsBinauralEffectParams {
            hrtf: self.hrtf.inner,
            order: order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsBinauralEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    /// Resets the internal processing state of an Ambisonics binaural effect.
    pub fn reset(&mut self) {
        unsafe {
            ffi::iplAmbisonicsBinauralEffectReset(self.inner);
        }
    }
}

unsafe impl Send for AmbisonicsBinauralEffect {}

impl Clone for AmbisonicsBinauralEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsBinauralEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
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

/// Applies a rotation to an Ambisonics audio buffer.
///
/// The input buffer is assumed to describe a sound field in “world space”. The output buffer is then the same sound field, but expressed relative to the listener’s orientation.
pub struct AmbisonicsRotationEffect {
    pub(crate) inner: ffi::IPLAmbisonicsRotationEffect,

    context: Context,
}

impl AmbisonicsRotationEffect {
    /// Creates an Ambisonics rotation effect.
    pub fn new(
        context: &Context,
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

        Ok(Self {
            inner: effect,
            context: context.clone(),
        })
    }

    /// Applies an Ambisonics rotation effect to an audio buffer.
    ///
    /// This effect CANNOT be applied in-place.
    pub fn apply(&mut self, orientation: Orientation, order: u8, in_: &Buffer, out: &mut Buffer) {
        let params = ffi::IPLAmbisonicsRotationEffectParams {
            orientation: orientation.into(),
            order: order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsRotationEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    /// Resets the internal processing state of an Ambisonics rotation effect.
    pub fn reset(&mut self) {
        unsafe {
            ffi::iplAmbisonicsRotationEffectReset(self.inner);
        }
    }
}

unsafe impl Send for AmbisonicsRotationEffect {}

impl Clone for AmbisonicsRotationEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsRotationEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
        }
    }
}

impl Drop for AmbisonicsRotationEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAmbisonicsRotationEffectRelease(&mut self.inner);
        }
    }
}

/// Applies a rotation to an Ambisonics audio buffer, then decodes it using panning or binaural rendering.
///
/// This is essentially an Ambisonics rotate effect followed by either an Ambisonics panning effect or an Ambisonics binaural effect.
pub struct AmbisonicsDecodeEffect {
    pub(crate) inner: ffi::IPLAmbisonicsDecodeEffect,

    context: Context,
    hrtf: Hrtf,
}

impl AmbisonicsDecodeEffect {
    /// Creates an Ambisonics rotation effect.
    pub fn new(
        context: &Context,
        sample_rate: u32,
        frame_length: u32,
        speaker_layout: SpeakerLayout,
        hrtf: &Hrtf,
        maximum_order: u8,
    ) -> Result<Self, Error> {
        let audio_settings = ffi::IPLAudioSettings {
            samplingRate: sample_rate as i32,
            frameSize: frame_length as i32,
        };
        let effect_settings = ffi::IPLAmbisonicsDecodeEffectSettings {
            speakerLayout: speaker_layout.into(),
            hrtf: hrtf.inner,
            maxOrder: maximum_order as i32,
        };
        let mut effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsDecodeEffectCreate(
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
            context: context.clone(),
            hrtf: hrtf.clone(),
        })
    }

    /// Applies an Ambisonics decode effect to an audio buffer.
    ///
    /// This effect CANNOT be applied in-place.
    pub fn apply(
        &mut self,
        orientation: Orientation,
        order: u8,
        binaural: bool,
        in_: &Buffer,
        out: &mut Buffer,
    ) {
        let params = ffi::IPLAmbisonicsDecodeEffectParams {
            hrtf: self.hrtf.inner,
            orientation: orientation.into(),
            order: order as i32,
            binaural: binaural.into(),
        };

        unsafe {
            ffi::iplAmbisonicsDecodeEffectApply(self.inner, &params, &in_.inner, &mut out.inner);
        }
    }

    /// Resets the internal processing state of an Ambisonics rotation effect.
    pub fn reset(&mut self) {
        unsafe {
            ffi::iplAmbisonicsDecodeEffectReset(self.inner);
        }
    }
}

unsafe impl Send for AmbisonicsDecodeEffect {}

impl Clone for AmbisonicsDecodeEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsDecodeEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
            hrtf: self.hrtf.clone(),
        }
    }
}

impl Drop for AmbisonicsDecodeEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAmbisonicsDecodeEffectRelease(&mut self.inner);
        }
    }
}

/// Filters and attenuates an audio signal based on various properties of the direct path between a point source and the listener.
pub struct DirectEffect {
    pub(crate) inner: ffi::IPLDirectEffect,

    context: Context,
}

impl DirectEffect {
    /// Creates a direct effect.
    pub fn new(
        context: &Context,
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

        Ok(DirectEffect {
            inner: effect,
            context: context.clone(),
        })
    }

    /// Applies a direct effect to an audio buffer.
    ///
    /// This effect CAN be applied in-place.
    pub fn apply(
        &mut self,
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

    /// Resets the internal processing state of a direct effect.
    pub fn reset(&mut self) {
        unsafe {
            ffi::iplDirectEffectReset(self.inner);
        }
    }
}

unsafe impl Send for DirectEffect {}

impl Clone for DirectEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplDirectEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
        }
    }
}

impl Drop for DirectEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplDirectEffectRelease(&mut self.inner);
        }
    }
}

/// Modes of applying transmission effects.
pub enum TransmissionType {
    /// Transmission is frequency-independent.
    FrequencyIndependent,

    /// Transmission is frequency-dependent.
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
