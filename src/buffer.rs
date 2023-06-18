use glam::Vec3;

use crate::ffi;

pub struct Buffer {
    pub(crate) inner: ffi::IPLAudioBuffer,

    pub(crate) data: Vec<Vec<f32>>,
    pub(crate) _data_ptrs: Vec<*mut f32>,
}

impl Buffer {
    pub fn data(&mut self) -> &mut Vec<Vec<f32>> {
        &mut self.data
    }

    pub fn channels(&self) -> u16 {
        self.inner.numChannels as u16
    }

    pub fn samples(&self) -> u32 {
        self.inner.numSamples as u32
    }
}

impl From<Vec<Vec<f32>>> for Buffer {
    fn from(mut value: Vec<Vec<f32>>) -> Self {
        let mut data_ptrs = value
            .iter_mut()
            .map(|data| data.as_mut_ptr())
            .collect::<Vec<_>>();

        Self {
            inner: ffi::IPLAudioBuffer {
                numChannels: value.len() as i32,
                numSamples: value.first().unwrap().len() as i32,
                data: data_ptrs.as_mut_ptr(),
            },
            data: value,
            _data_ptrs: data_ptrs,
        }
    }
}

unsafe impl Send for Buffer {}

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
