use glam::Vec3;

use context::Context;
use ffi;
use geometry::Orientation;
use hrtf::Hrtf;

trait Effect<T> {
    fn apply(&self, params: T);

    fn reset(&self);
}

/// Pans a single-channel point source to a multi-channel speaker layout based
/// on the 3D position of the source relative to the listener.
pub struct PanningEffect {
    pub(crate) inner: ffi::IPLPanningEffect,

    pub(crate) context: Context,
}

pub struct PanningEffectParams {
    pub direction: Vec3,
}

impl Effect<PanningEffectParams> for PanningEffect {
    fn apply(&self, params: PanningEffectParams) {
        let mut params = ffi::IPLPanningEffectParams {
            direction: params.direction.into(),
        };

        unsafe {
            ffi::iplPanningEffectApply(
                self.inner,
                &mut params,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
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

/// Spatializes a point source using an HRTF, based on the 3D position of the
/// source relative to the listener.
///
/// The source audio can be 1- or 2-channel; in either case all input channels
/// are spatialized from the same position.
pub struct BinauralEffect {
    pub(crate) inner: ffi::IPLBinauralEffect,

    pub(crate) context: Context,
    pub(crate) hrtf: Hrtf,
}

pub struct BinauralEffectParams {
    pub direction: Vec3,
    pub interpolation: HrtfInterpolation,
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
    fn apply(&self, params: BinauralEffectParams) {
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
                std::ptr::null_mut(),
                std::ptr::null_mut(),
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
    pub(crate) inner: ffi::IPLVirtualSurroundEffect,

    pub(crate) context: Context,
    pub(crate) hrtf: Hrtf,
}

impl Effect<()> for VirtualSurroundEffect {
    fn apply(&self, _params: ()) {
        let mut params = ffi::IPLVirtualSurroundEffectParams {
            hrtf: self.hrtf.inner,
        };

        unsafe {
            ffi::iplVirtualSurroundEffectApply(
                self.inner,
                &mut params,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
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

/// Encodes a point source into Ambisonics.
///
/// Given a point source with some direction relative to the listener, this
/// effect generates an Ambisonic audio buffer that approximates a point source
/// in the given direction. This allows multiple point sources and ambiences to
/// mixed to a single Ambisonics buffer before being spatialized.
pub struct AmbisonicsEncodeEffect {
    pub(crate) inner: ffi::IPLAmbisonicsEncodeEffect,

    pub(crate) context: Context,
}

pub struct AmbisonicsEncodeEffectParams {
    pub direction: Vec3,
    pub order: u8,
}

impl Effect<AmbisonicsEncodeEffectParams> for AmbisonicsEncodeEffect {
    fn apply(&self, params: AmbisonicsEncodeEffectParams) {
        let mut params = ffi::IPLAmbisonicsEncodeEffectParams {
            direction: params.direction.into(),
            order: params.order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsEncodeEffectApply(
                self.inner,
                &mut params,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
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

/// Renders Ambisonic audio by panning it to a standard speaker layout.
///
/// This involves calculating signals to emit from each speaker so as to
/// approximate the Ambisonic sound field.
pub struct AmbisonicsPanningEffect {
    pub(crate) inner: ffi::IPLAmbisonicsPanningEffect,

    pub(crate) context: Context,
}

pub struct AmbisonicsPanningEffectParams {
    pub order: u8,
}

impl Effect<AmbisonicsPanningEffectParams> for AmbisonicsPanningEffect {
    fn apply(&self, params: AmbisonicsPanningEffectParams) {
        let mut params = ffi::IPLAmbisonicsPanningEffectParams {
            order: params.order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsPanningEffectApply(
                self.inner,
                &mut params,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
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

/// Renders Ambisonic audio using HRTF-based binaural rendering.
///
/// This results in more immersive spatialization of the Ambisonic audio as
/// compared to using an Ambisonics panning effect, at the cost of slightly
/// increased CPU usage.
pub struct AmbisonicsBinauralEffect {
    pub(crate) inner: ffi::IPLAmbisonicsBinauralEffect,

    pub(crate) context: Context,
    pub(crate) hrtf: Hrtf,
}

pub struct AmbisonicsBinauralEffectParams {
    pub order: u8,
}

impl Effect<AmbisonicsBinauralEffectParams> for AmbisonicsBinauralEffect {
    fn apply(&self, params: AmbisonicsBinauralEffectParams) {
        let mut params = ffi::IPLAmbisonicsBinauralEffectParams {
            hrtf: self.hrtf.inner,
            order: params.order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsBinauralEffectApply(
                self.inner,
                &mut params,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
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

/// Applies a rotation to an Ambisonics audio buffer.
///
/// The input buffer is assumed to describe a sound field in “world space”. The
/// output buffer is then the same sound field, but expressed relative to the
/// listener’s orientation.
pub struct AmbisonicsRotationEffect {
    pub(crate) inner: ffi::IPLAmbisonicsRotationEffect,

    pub(crate) context: Context,
}

pub struct AmbisonicsRotationEffectParams {
    pub orientation: Orientation,
    pub order: u8,
}

impl Effect<AmbisonicsRotationEffectParams> for AmbisonicsRotationEffect {
    fn apply(&self, params: AmbisonicsRotationEffectParams) {
        let mut params = ffi::IPLAmbisonicsRotationEffectParams {
            orientation: params.orientation.into(),
            order: params.order as i32,
        };

        unsafe {
            ffi::iplAmbisonicsRotationEffectApply(
                self.inner,
                &mut params,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
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

/// Applies a rotation to an Ambisonics audio buffer, then decodes it using
/// panning or binaural rendering.
///
/// This is essentially an Ambisonics rotate effect followed by either an
/// Ambisonics panning effect or an Ambisonics binaural effect.
pub struct AmbisonicsDecodeEffect {
    pub(crate) inner: ffi::IPLAmbisonicsDecodeEffect,

    pub(crate) context: Context,
    pub(crate) hrtf: Hrtf,
}

pub struct AmbisonicsDecodeEffectParams {
    pub orientation: Orientation,
    pub order: u8,
    pub binaural: bool,
}

impl Effect<AmbisonicsDecodeEffectParams> for AmbisonicsDecodeEffect {
    fn apply(&self, params: AmbisonicsDecodeEffectParams) {
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
                std::ptr::null_mut(),
                std::ptr::null_mut(),
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
