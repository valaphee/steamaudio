pub mod buffer;
pub mod context;
pub mod effect;
pub mod error;
pub mod geometry;
pub mod hrtf;

pub mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod prelude {
    pub use crate::{
        buffer::{Buffer, SpeakerLayout},
        context::Context,
        effect::{
            AmbisonicsBinauralEffect, AmbisonicsDecodeEffect, AmbisonicsEncodeEffect,
            AmbisonicsPanningEffect, AmbisonicsRotationEffect, BinauralEffect, DirectEffect,
            HrtfInterpolation, PanningEffect, VirtualSurroundEffect,
        },
        error::Error,
        geometry::Orientation,
        hrtf::{Hrtf, HrtfType},
    };
}
