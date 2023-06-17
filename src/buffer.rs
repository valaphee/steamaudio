use glam::Vec3;

use ffi;

/// Describes a standard or custom speaker layout.
pub enum SpeakerLayout {
    /// Mono.
    Mono,

    /// Stereo (left, right).
    Stereo,

    /// Front left, front right, rear left, rear right
    Quadraphonic,

    /// Front left, front right, front center, LFE, rear left, rear right.
    Surround5_1,

    /// Front left, front right, front center, LFE, rear left, rear right, side
    /// left, side right.
    Surround7_1,

    /// User-defined speaker layout.
    Custom(Vec<Vec3>),
}

impl From<SpeakerLayout> for ffi::IPLSpeakerLayout {
    fn from(value: SpeakerLayout) -> ffi::IPLSpeakerLayout {
        match value {
            SpeakerLayout::Mono => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_MONO,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            SpeakerLayout::Stereo => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_STEREO,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            SpeakerLayout::Quadraphonic => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_QUADRAPHONIC,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            SpeakerLayout::Surround5_1 => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_SURROUND_5_1,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            SpeakerLayout::Surround7_1 => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_SURROUND_7_1,
                numSpeakers: 0,
                speakers: std::ptr::null_mut(),
            },
            SpeakerLayout::Custom(speakers) => ffi::IPLSpeakerLayout {
                type_: ffi::IPLSpeakerLayoutType_IPL_SPEAKERLAYOUTTYPE_CUSTOM,
                numSpeakers: speakers.len() as i32,
                speakers: speakers
                    .iter()
                    .map(|speaker| speaker.into())
                    .collect::<Vec<_>>()
                    .as_mut_ptr(),
            },
        }
    }
}
