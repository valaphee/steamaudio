pub mod ambisonics_binaural_effect;
pub mod ambisonics_decode_effect;
pub mod ambisonics_encode_effect;
pub mod ambisonics_panning_effect;
pub mod ambisonics_rotation_effect;
pub mod binaural_effect;
pub mod buffer;
pub mod context;
pub mod direct_effect;
pub mod error;
pub mod geometry;
pub mod hrtf;
pub mod panning_effect;
pub mod virtual_surround_effect;

pub mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod prelude {
    pub use crate::ambisonics_binaural_effect::AmbisonicsBinauralEffect;
    pub use crate::ambisonics_decode_effect::AmbisonicsDecodeEffect;
    pub use crate::ambisonics_encode_effect::AmbisonicsEncodeEffect;
    pub use crate::ambisonics_panning_effect::AmbisonicsPanningEffect;
    pub use crate::ambisonics_rotation_effect::AmbisonicsRotationEffect;
    pub use crate::binaural_effect::{BinauralEffect, HrtfInterpolation};
    pub use crate::buffer::{Buffer, SpeakerLayout};
    pub use crate::context::Context;
    pub use crate::direct_effect::DirectEffect;
    pub use crate::error::Error;
    pub use crate::hrtf::{Hrtf, HrtfType};
    pub use crate::geometry::Orientation;
    pub use crate::panning_effect::PanningEffect;
    pub use crate::virtual_surround_effect::VirtualSurroundEffect;
}
