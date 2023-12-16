use glam::Vec3;

use crate::{
    buffer::{Buffer, SpeakerLayout},
    context::Context,
    error::check,
    ffi,
    geometry::Orientation,
    hrtf::Hrtf,
    simulation::Source,
};

impl Context {
    /// Creates a panning effect.
    pub fn create_panning_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        speaker_layout: SpeakerLayout,
    ) -> crate::error::Result<PanningEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut panning_effect_settings = ffi::IPLPanningEffectSettings {
            speakerLayout: speaker_layout.into(),
        };
        let mut panning_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplPanningEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut panning_effect_settings,
                    &mut panning_effect,
                ),
                PanningEffect {
                    inner: panning_effect,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates a binaural effect.
    pub fn create_binaural_effect(
        &self,
        hrtf: &Hrtf,
        sampling_rate: u32,
        frame_size: u32,
    ) -> crate::error::Result<BinauralEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut binaural_effect_settings = ffi::IPLBinauralEffectSettings { hrtf: hrtf.inner };
        let mut binaural_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplBinauralEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut binaural_effect_settings,
                    &mut binaural_effect,
                ),
                BinauralEffect {
                    inner: binaural_effect,
                    context: self.clone(),
                    hrtf: hrtf.clone(),
                },
            )
        }
    }

    /// Creates a virtual surround effect.
    pub fn create_virtual_surround_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        speaker_layout: SpeakerLayout,
        hrtf: &Hrtf,
    ) -> crate::error::Result<VirtualSurroundEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut virtual_surround_effect_settings = ffi::IPLVirtualSurroundEffectSettings {
            speakerLayout: speaker_layout.into(),
            hrtf: hrtf.inner,
        };
        let mut virtual_surround_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplVirtualSurroundEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut virtual_surround_effect_settings,
                    &mut virtual_surround_effect,
                ),
                VirtualSurroundEffect {
                    inner: virtual_surround_effect,
                    context: self.clone(),
                    hrtf: hrtf.clone(),
                },
            )
        }
    }

    /// Creates an Ambisonics encode effect.
    pub fn create_ambisonics_encode_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        maximum_order: u8,
    ) -> crate::error::Result<AmbisonicsEncodeEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut ambisonics_encode_effect_settings = ffi::IPLAmbisonicsEncodeEffectSettings {
            maxOrder: maximum_order as i32,
        };
        let mut ambisonics_encode_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsEncodeEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut ambisonics_encode_effect_settings,
                    &mut ambisonics_encode_effect,
                ),
                AmbisonicsEncodeEffect {
                    inner: ambisonics_encode_effect,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates an Ambisonics panning effect.
    pub fn create_ambisonics_panning_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        speaker_layout: SpeakerLayout,
        maximum_order: u8,
    ) -> crate::error::Result<AmbisonicsPanningEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut ambisonics_panning_effect_settings = ffi::IPLAmbisonicsPanningEffectSettings {
            speakerLayout: speaker_layout.into(),
            maxOrder: maximum_order as i32,
        };
        let mut ambisonics_panning_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsPanningEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut ambisonics_panning_effect_settings,
                    &mut ambisonics_panning_effect,
                ),
                AmbisonicsPanningEffect {
                    inner: ambisonics_panning_effect,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates an Ambisonics binaural effect.
    pub fn create_ambisonics_binaural_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        hrtf: &Hrtf,
        maximum_order: u8,
    ) -> crate::error::Result<AmbisonicsBinauralEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut ambisonics_binaural_effect_settings = ffi::IPLAmbisonicsBinauralEffectSettings {
            hrtf: hrtf.inner,
            maxOrder: maximum_order as i32,
        };
        let mut ambisonics_binaural_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsBinauralEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut ambisonics_binaural_effect_settings,
                    &mut ambisonics_binaural_effect,
                ),
                AmbisonicsBinauralEffect {
                    inner: ambisonics_binaural_effect,
                    context: self.clone(),
                    hrtf: hrtf.clone(),
                },
            )
        }
    }

    /// Creates an Ambisonics rotation effect.
    pub fn create_ambisonics_rotation_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        maximum_order: u8,
    ) -> crate::error::Result<AmbisonicsRotationEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut ambisonics_rotation_effect_settings = ffi::IPLAmbisonicsRotationEffectSettings {
            maxOrder: maximum_order as i32,
        };
        let mut ambisonics_rotation_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsRotationEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut ambisonics_rotation_effect_settings,
                    &mut ambisonics_rotation_effect,
                ),
                AmbisonicsRotationEffect {
                    inner: ambisonics_rotation_effect,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates an Ambisonics decode effect.
    pub fn create_ambisonics_decode_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        speaker_layout: SpeakerLayout,
        hrtf: &Hrtf,
        maximum_order: u8,
    ) -> crate::error::Result<AmbisonicsDecodeEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut ambisonics_decode_effect_settings = ffi::IPLAmbisonicsDecodeEffectSettings {
            speakerLayout: speaker_layout.into(),
            hrtf: hrtf.inner,
            maxOrder: maximum_order as i32,
        };
        let mut ambisonics_decode_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsDecodeEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut ambisonics_decode_effect_settings,
                    &mut ambisonics_decode_effect,
                ),
                AmbisonicsDecodeEffect {
                    inner: ambisonics_decode_effect,
                    context: self.clone(),
                    hrtf: hrtf.clone(),
                },
            )
        }
    }

    /// Creates a direct effect.
    pub fn create_direct_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        channels: u16,
    ) -> crate::error::Result<DirectEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut direct_effect_settings = ffi::IPLDirectEffectSettings {
            numChannels: channels as i32,
        };
        let mut direct_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplDirectEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut direct_effect_settings,
                    &mut direct_effect,
                ),
                DirectEffect {
                    inner: direct_effect,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates a reflection effect.
    pub fn create_reflection_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        channels: u16,
    ) -> crate::error::Result<ReflectionEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut reflection_effect_settings = ffi::IPLReflectionEffectSettings {
            type_: ffi::IPLReflectionEffectType_IPL_REFLECTIONEFFECTTYPE_CONVOLUTION,
            irSize: (2 * sampling_rate) as i32,
            numChannels: channels as i32,
        };
        let mut reflection_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplReflectionEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut reflection_effect_settings,
                    &mut reflection_effect,
                ),
                ReflectionEffect {
                    inner: reflection_effect,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates a path effect.
    pub fn create_path_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        maximum_order: u8,
    ) -> crate::error::Result<PathEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut path_effect_settings = ffi::IPLPathEffectSettings {
            maxOrder: maximum_order as i32,
            spatialize: 0,
            speakerLayout: ffi::IPLSpeakerLayout {
                type_: 0,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            hrtf: std::ptr::null_mut(),
        };
        let mut path_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplPathEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut path_effect_settings,
                    &mut path_effect,
                ),
                PathEffect {
                    inner: path_effect,
                    context: self.clone(),
                },
            )
        }
    }
}

pub trait Effect<T> {
    fn apply(&self, params: T, in_: &Buffer, out: &mut Buffer);

    fn reset(&self);
}

/// Pans a single-channel point source to a multi-channel speaker layout based
/// on the 3D position of the source relative to the listener.
pub struct PanningEffect {
    inner: ffi::IPLPanningEffect,

    context: Context,
}

/// Parameters for applying a panning effect to an audio buffer.
pub struct PanningEffectParams {
    /// Unit vector pointing from the listener towards the source.
    pub direction: Vec3,
}

impl Effect<PanningEffectParams> for PanningEffect {
    fn apply(&self, params: PanningEffectParams, in_: &Buffer, out: &mut Buffer) {
        let mut params = ffi::IPLPanningEffectParams {
            direction: params.direction.into(),
        };

        unsafe {
            ffi::iplPanningEffectApply(
                self.inner,
                &mut params,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplPanningEffectReset(self.inner);
        }
    }
}

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

unsafe impl Send for PanningEffect {}

unsafe impl Sync for PanningEffect {}

/// Spatializes a point source using an HRTF, based on the 3D position of the
/// source relative to the listener.
///
/// The source audio can be 1- or 2-channel; in either case all input channels
/// are spatialized from the same position.
pub struct BinauralEffect {
    inner: ffi::IPLBinauralEffect,

    context: Context,
    hrtf: Hrtf,
}

/// Parameters for applying a binaural effect to an audio buffer.
pub struct BinauralEffectParams {
    /// Unit vector pointing from the listener towards the source.
    pub direction: Vec3,

    /// The interpolation technique to use.
    pub interpolation: HrtfInterpolation,

    /// Amount to blend input audio with spatialized audio. When set to 0,
    /// output audio is not spatialized at all and is close to input audio.
    /// If set to 1, output audio is fully spatialized.
    pub spatial_blend: f32,
}

/// Techniques for interpolating HRTF data.
///
/// This is used when rendering a point source whose position relative to the
/// listener is not contained in the measured HRTF data.
pub enum HrtfInterpolation {
    /// Nearest-neighbor filtering, i.e., no interpolation.
    ///
    /// Selects the measurement location that is closest to the source’s actual
    /// location.
    Nearest,

    /// Bilinear filtering.
    ///
    /// Incurs a relatively high CPU overhead as compared to nearest-neighbor
    /// filtering, so use this for sounds where it has a significant benefit.
    /// Typically, bilinear filtering is most useful for wide-band noise-like
    /// sounds, such as radio static, mechanical noise, fire, etc.
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

impl Effect<BinauralEffectParams> for BinauralEffect {
    fn apply(&self, params: BinauralEffectParams, in_: &Buffer, out: &mut Buffer) {
        let mut params = ffi::IPLBinauralEffectParams {
            direction: params.direction.into(),
            interpolation: params.interpolation.into(),
            spatialBlend: params.spatial_blend,
            hrtf: self.hrtf.inner,
            peakDelays: std::ptr::null_mut(),
        };

        unsafe {
            ffi::iplBinauralEffectApply(
                self.inner,
                &mut params,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplBinauralEffectReset(self.inner);
        }
    }
}

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

unsafe impl Send for BinauralEffect {}

unsafe impl Sync for BinauralEffect {}

/// Spatializes multi-channel speaker-based audio (e.g., stereo, quadraphonic,
/// 5.1, or 7.1) using HRTF-based binaural rendering.
///
/// The audio signal for each speaker is spatialized from a point in space
/// corresponding to the speaker’s location. This allows users to experience a
/// surround sound mix over regular stereo headphones.
///
/// Virtual surround is also a fast way to get approximate binaural rendering.
/// All sources can be panned to some surround format (say, 7.1). After the
/// sources are mixed, the mix can be rendered using virtual surround. This can
/// reduce CPU usage, at the cost of spatialization accuracy.
pub struct VirtualSurroundEffect {
    inner: ffi::IPLVirtualSurroundEffect,

    context: Context,
    hrtf: Hrtf,
}

impl Effect<()> for VirtualSurroundEffect {
    fn apply(&self, _params: (), in_: &Buffer, out: &mut Buffer) {
        let mut params = ffi::IPLVirtualSurroundEffectParams {
            hrtf: self.hrtf.inner,
        };

        unsafe {
            ffi::iplVirtualSurroundEffectApply(
                self.inner,
                &mut params,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplVirtualSurroundEffectReset(self.inner);
        }
    }
}

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

unsafe impl Send for VirtualSurroundEffect {}

unsafe impl Sync for VirtualSurroundEffect {}

/// Encodes a point source into Ambisonics.
///
/// Given a point source with some direction relative to the listener, this
/// effect generates an Ambisonic audio buffer that approximates a point source
/// in the given direction. This allows multiple point sources and ambiences to
/// mixed to a single Ambisonics buffer before being spatialized.
pub struct AmbisonicsEncodeEffect {
    inner: ffi::IPLAmbisonicsEncodeEffect,

    context: Context,
}

/// Parameters for applying an Ambisonics encode effect to an audio buffer.
pub struct AmbisonicsEncodeEffectParams {
    /// Vector pointing from the listener towards the source. Need not be
    /// normalized; Steam Audio will automatically normalize this vector. If
    /// a zero-length vector is passed, the output will be order 0
    /// (omnidirectional).
    pub direction: Vec3,

    /// Ambisonic order of the output buffer. May be less than the \c maxOrder
    /// specified when creating the effect, in which case the effect will
    /// generate fewer output channels, reducing CPU usage.
    pub order: u8,
}

impl Effect<AmbisonicsEncodeEffectParams> for AmbisonicsEncodeEffect {
    fn apply(&self, params: AmbisonicsEncodeEffectParams, in_: &Buffer, out: &mut Buffer) {
        let mut params = ffi::IPLAmbisonicsEncodeEffectParams {
            direction: params.direction.into(),
            order: params.order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsEncodeEffectApply(
                self.inner,
                &mut params,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplAmbisonicsEncodeEffectReset(self.inner);
        }
    }
}

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

unsafe impl Send for AmbisonicsEncodeEffect {}

unsafe impl Sync for AmbisonicsEncodeEffect {}

/// Renders Ambisonic audio by panning it to a standard speaker layout.
///
/// This involves calculating signals to emit from each speaker so as to
/// approximate the Ambisonic sound field.
pub struct AmbisonicsPanningEffect {
    inner: ffi::IPLAmbisonicsPanningEffect,

    context: Context,
}

/// Parameters for applying an Ambisonics panning effect to an audio buffer.
pub struct AmbisonicsPanningEffectParams {
    /// Ambisonic order of the input buffer. May be less than the \c maxOrder
    /// specified when creating the effect, in which case the effect will
    /// process fewer input channels, reducing CPU usage.
    pub order: u8,
}

impl Effect<AmbisonicsPanningEffectParams> for AmbisonicsPanningEffect {
    fn apply(&self, params: AmbisonicsPanningEffectParams, in_: &Buffer, out: &mut Buffer) {
        let mut params = ffi::IPLAmbisonicsPanningEffectParams {
            order: params.order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsPanningEffectApply(
                self.inner,
                &mut params,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplAmbisonicsPanningEffectReset(self.inner);
        }
    }
}

impl Clone for AmbisonicsPanningEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplAmbisonicsPanningEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
        }
    }
}

impl Drop for AmbisonicsPanningEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplAmbisonicsPanningEffectRelease(&mut self.inner);
        }
    }
}

unsafe impl Send for AmbisonicsPanningEffect {}

unsafe impl Sync for AmbisonicsPanningEffect {}

/// Renders Ambisonic audio using HRTF-based binaural rendering.
///
/// This results in more immersive spatialization of the Ambisonic audio as
/// compared to using an Ambisonics panning effect, at the cost of slightly
/// increased CPU usage.
pub struct AmbisonicsBinauralEffect {
    inner: ffi::IPLAmbisonicsBinauralEffect,

    context: Context,
    hrtf: Hrtf,
}

/// Parameters for applying an Ambisonics binaural effect to an audio buffer.
pub struct AmbisonicsBinauralEffectParams {
    /// Ambisonic order of the input buffer. May be less than the \c maxOrder
    /// specified when creating the effect, in which case the effect will
    /// process fewer input channels, reducing CPU usage.
    pub order: u8,
}

impl Effect<AmbisonicsBinauralEffectParams> for AmbisonicsBinauralEffect {
    fn apply(&self, params: AmbisonicsBinauralEffectParams, in_: &Buffer, out: &mut Buffer) {
        let mut params = ffi::IPLAmbisonicsBinauralEffectParams {
            hrtf: self.hrtf.inner,
            order: params.order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsBinauralEffectApply(
                self.inner,
                &mut params,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplAmbisonicsBinauralEffectReset(self.inner);
        }
    }
}

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

unsafe impl Send for AmbisonicsBinauralEffect {}

unsafe impl Sync for AmbisonicsBinauralEffect {}

/// Applies a rotation to an Ambisonics audio buffer.
///
/// The input buffer is assumed to describe a sound field in “world space”. The
/// output buffer is then the same sound field, but expressed relative to the
/// listener’s orientation.
pub struct AmbisonicsRotationEffect {
    inner: ffi::IPLAmbisonicsRotationEffect,

    context: Context,
}

/// Parameters for applying an Ambisonics rotation effect to an audio buffer.
pub struct AmbisonicsRotationEffectParams {
    /// The orientation of the listener.
    pub orientation: Orientation,

    /// Ambisonic order of the input and output buffers. May be less than the \c
    /// maxOrder specified when creating the effect, in which case the
    /// effect will process fewer channels, reducing CPU usage.
    pub order: u8,
}

impl Effect<AmbisonicsRotationEffectParams> for AmbisonicsRotationEffect {
    fn apply(&self, params: AmbisonicsRotationEffectParams, in_: &Buffer, out: &mut Buffer) {
        let mut params = ffi::IPLAmbisonicsRotationEffectParams {
            orientation: params.orientation.into(),
            order: params.order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsRotationEffectApply(
                self.inner,
                &mut params,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplAmbisonicsRotationEffectReset(self.inner);
        }
    }
}

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

unsafe impl Send for AmbisonicsRotationEffect {}

unsafe impl Sync for AmbisonicsRotationEffect {}

/// Applies a rotation to an Ambisonics audio buffer, then decodes it using
/// panning or binaural rendering.
///
/// This is essentially an Ambisonics rotate effect followed by either an
/// Ambisonics panning effect or an Ambisonics binaural effect.
pub struct AmbisonicsDecodeEffect {
    inner: ffi::IPLAmbisonicsDecodeEffect,

    context: Context,
    hrtf: Hrtf,
}

/// Parameters for applying an Ambisonics decode effect to an audio buffer.
pub struct AmbisonicsDecodeEffectParams {
    /// The orientation of the listener.
    pub orientation: Orientation,

    /// Ambisonic order of the input buffer. May be less than the \c maxOrder
    /// specified when creating the effect, in which case the effect will
    /// process fewer input channels, reducing CPU usage.
    pub order: u8,

    /// Whether to use binaural rendering or panning.
    pub binaural: bool,
}

impl Effect<AmbisonicsDecodeEffectParams> for AmbisonicsDecodeEffect {
    fn apply(&self, params: AmbisonicsDecodeEffectParams, in_: &Buffer, out: &mut Buffer) {
        let mut params = ffi::IPLAmbisonicsDecodeEffectParams {
            order: params.order as i32,
            hrtf: self.hrtf.inner,
            orientation: params.orientation.into(),
            binaural: params.binaural.into(),
        };

        unsafe {
            ffi::iplAmbisonicsDecodeEffectApply(
                self.inner,
                &mut params,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplAmbisonicsDecodeEffectReset(self.inner);
        }
    }
}

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

unsafe impl Send for AmbisonicsDecodeEffect {}

unsafe impl Sync for AmbisonicsDecodeEffect {}

/// Filters and attenuates an audio signal based on various properties of the
/// direct path between a point source and the listener.
pub struct DirectEffect {
    inner: ffi::IPLDirectEffect,

    context: Context,
}

impl Effect<&Source> for DirectEffect {
    fn apply(&self, params: &Source, in_: &Buffer, out: &mut Buffer) {
        unsafe {
            let mut simulation_outputs = std::mem::zeroed();

            ffi::iplSourceGetOutputs(
                params.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                &mut simulation_outputs,
            );
            simulation_outputs.direct.flags = params.inputs.borrow().directFlags;
            ffi::iplDirectEffectApply(
                self.inner,
                &mut simulation_outputs.direct,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplDirectEffectReset(self.inner);
        }
    }
}

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

unsafe impl Send for DirectEffect {}

unsafe impl Sync for DirectEffect {}

/// Applies the result of physics-based reflections simulation to an audio
/// buffer. The result is encoded in Ambisonics, and can be decoded using an
/// Ambisonics decode effect
pub struct ReflectionEffect {
    inner: ffi::IPLReflectionEffect,

    context: Context,
}

impl Effect<&Source> for ReflectionEffect {
    fn apply(&self, params: &Source, in_: &Buffer, out: &mut Buffer) {
        unsafe {
            let mut simulation_outputs = std::mem::zeroed();

            ffi::iplSourceGetOutputs(
                params.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_REFLECTIONS,
                &mut simulation_outputs,
            );
            simulation_outputs.reflections.numChannels = 4;
            simulation_outputs.reflections.irSize = 2 * 44100;
            ffi::iplReflectionEffectApply(
                self.inner,
                &mut simulation_outputs.reflections,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
                std::ptr::null_mut(),
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplReflectionEffectReset(self.inner);
        }
    }
}

impl Clone for ReflectionEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplReflectionEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
        }
    }
}

impl Drop for ReflectionEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplReflectionEffectRelease(&mut self.inner);
        }
    }
}

unsafe impl Send for ReflectionEffect {}

unsafe impl Sync for ReflectionEffect {}

/// Applies the result of simulating sound paths from the source to the
/// listener. Multiple paths that sound can take as it propagates from the
/// source to the listener are combined into an Ambisonic sound field.
pub struct PathEffect {
    inner: ffi::IPLPathEffect,

    context: Context,
}

impl Effect<&Source> for PathEffect {
    fn apply(&self, params: &Source, in_: &Buffer, out: &mut Buffer) {
        unsafe {
            let mut simulation_outputs = std::mem::zeroed();

            ffi::iplSourceGetOutputs(
                params.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                &mut simulation_outputs,
            );
            ffi::iplPathEffectApply(
                self.inner,
                &mut simulation_outputs.pathing,
                std::mem::transmute(&in_.inner),
                &mut out.inner,
            );
        }
    }

    fn reset(&self) {
        unsafe {
            ffi::iplPathEffectReset(self.inner);
        }
    }
}

impl Clone for PathEffect {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplPathEffectRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
        }
    }
}

impl Drop for PathEffect {
    fn drop(&mut self) {
        unsafe {
            ffi::iplPathEffectRelease(&mut self.inner);
        }
    }
}

unsafe impl Send for PathEffect {}

unsafe impl Sync for PathEffect {}
