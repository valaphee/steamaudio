use crate::ffi;
use crate::prelude::*;
use glam::{Quat, Vec3};
use crate::ffi::iplCalculateRelativeDirection;

impl From<ffi::IPLVector3> for Vec3 {
    fn from(value: ffi::IPLVector3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Vec3> for ffi::IPLVector3 {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<&Vec3> for ffi::IPLVector3 {
    fn from(value: &Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

pub struct Orientation {
    pub translation: Vec3,
    pub rotation: Quat,
}

impl Orientation {
    pub fn relative_direction(&self, context: Context, to: Vec3) -> Vec3 {
        unsafe {
            iplCalculateRelativeDirection(context.inner, to.into(), self.translation.into(), (self.rotation * Vec3::NEG_Z).into(), (self.rotation * Vec3::Y).into()).into()
        }
    }
}

impl From<Orientation> for ffi::IPLCoordinateSpace3 {
    fn from(value: Orientation) -> Self {
        Self {
            right: (value.rotation * Vec3::X).into(),
            up: (value.rotation * Vec3::Y).into(),
            ahead: (value.rotation * Vec3::NEG_Z).into(),
            origin: value.translation.into(),
        }
    }
}
