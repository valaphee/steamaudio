use glam::{Mat4, Quat, Vec3};

use crate::{context::Context, ffi};

impl Context {
    /// Calculates the relative direction from the listener to a sound source.
    /// The returned direction vector is expressed in the listener's
    /// coordinate system.
    pub fn calculate_relative_direction(&self, source: Vec3, listener: Orientation) -> Vec3 {
        unsafe {
            ffi::iplCalculateRelativeDirection(
                self.inner,
                source.into(),
                listener.translation.into(),
                (listener.rotation * Vec3::NEG_Z).into(),
                (listener.rotation * Vec3::Y).into(),
            )
            .into()
        }
    }
}

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

impl From<Mat4> for ffi::IPLMatrix4x4 {
    fn from(value: Mat4) -> Self {
        Self {
            elements: [
                [
                    value.x_axis.x,
                    value.y_axis.x,
                    value.z_axis.x,
                    value.w_axis.x,
                ],
                [
                    value.x_axis.y,
                    value.y_axis.y,
                    value.z_axis.y,
                    value.w_axis.y,
                ],
                [
                    value.x_axis.z,
                    value.y_axis.z,
                    value.z_axis.z,
                    value.w_axis.z,
                ],
                [
                    value.x_axis.w,
                    value.y_axis.w,
                    value.z_axis.w,
                    value.w_axis.w,
                ],
            ],
        }
    }
}

impl From<&Mat4> for ffi::IPLMatrix4x4 {
    fn from(value: &Mat4) -> Self {
        Self {
            elements: [
                [
                    value.x_axis.x,
                    value.y_axis.x,
                    value.z_axis.x,
                    value.w_axis.x,
                ],
                [
                    value.x_axis.y,
                    value.y_axis.y,
                    value.z_axis.y,
                    value.w_axis.y,
                ],
                [
                    value.x_axis.z,
                    value.y_axis.z,
                    value.z_axis.z,
                    value.w_axis.z,
                ],
                [
                    value.x_axis.w,
                    value.y_axis.w,
                    value.z_axis.w,
                    value.w_axis.w,
                ],
            ],
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Orientation {
    pub translation: Vec3,
    pub rotation: Quat,
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
