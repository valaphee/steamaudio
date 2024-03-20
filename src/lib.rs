pub mod buffer;
pub mod context;
pub mod effect;
pub mod error;
pub mod geometry;
pub mod hrtf;
pub mod scene;
pub mod simulation;

#[cfg(feature = "fmod")]
pub mod fmod;

#[cfg(feature = "rodio")]
pub mod transform;

#[allow(
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals
)]
mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub fn ambisonics_channels(order: u8) -> u16 {
    (order as u16 + 1).pow(2)
}
