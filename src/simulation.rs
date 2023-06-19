use std::cell::RefCell;

use glam::Vec3;

use crate::{
    error::{check, Result},
    ffi,
    geometry::Orientation,
    scene::Scene,
};

/// Manages direct and indirect sound propagation simulation for multiple
/// sources. Your application will typically create one simulator object and use
/// it to run simulations with different source and listener parameters between
/// consecutive simulation runs. The simulator can also be reused across scene
/// changes.
pub struct Simulator {
    pub(crate) inner: ffi::IPLSimulator,
    pub(crate) shared_inputs: RefCell<ffi::IPLSimulationSharedInputs>,
}

impl Simulator {
    /// Specifies the scene within which all subsequent simulations should be
    /// run.
    pub fn set_scene(&mut self, scene: &Scene) {
        unsafe {
            ffi::iplSimulatorSetScene(self.inner, scene.inner);
        }
    }

    /// Commits changes to the scene or probe batches used for simulation.
    pub fn commit(&self) {
        unsafe {
            ffi::iplSimulatorCommit(self.inner);
        }
    }

    /// Specifies simulation parameters that are not associated with any
    /// particular source.
    pub fn set_listener(&mut self, listener: Orientation) {
        self.shared_inputs.get_mut().listener = listener.into();

        unsafe {
            ffi::iplSimulatorSetSharedInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT
                    | ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_REFLECTIONS
                    | ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_PATHING,
                self.shared_inputs.as_ptr(),
            );
        }
    }

    pub fn set_reflections(
        &mut self,
        rays: u32,
        bounces: u32,
        duration: f32,
        order: u8,
        irradiance_minimum_distance: f32,
    ) {
        let shared_inputs = self.shared_inputs.get_mut();
        shared_inputs.numRays = rays as i32;
        shared_inputs.numBounces = bounces as i32;
        shared_inputs.duration = duration;
        shared_inputs.order = order as i32;
        shared_inputs.irradianceMinDistance = irradiance_minimum_distance;

        unsafe {
            ffi::iplSimulatorSetSharedInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_REFLECTIONS,
                self.shared_inputs.as_ptr(),
            );
        }
    }

    /// Runs a direct simulation for all sources added to the simulator. This
    /// may include distance attenuation, air absorption, directivity,
    /// occlusion, and transmission.
    ///
    /// This function should not be called from the audio processing thread if
    /// occlusion and/or transmission are enabled.
    pub fn run_direct(&self) {
        unsafe {
            ffi::iplSimulatorRunDirect(self.inner);
        }
    }

    /// Runs a reflections simulation for all sources added to the simulator.
    ///
    /// This function can be CPU intensive, and should be called from a separate
    /// thread in order to not block either the audio processing thread or
    /// the game's main update thread.
    pub fn run_reflections(&self) {
        unsafe {
            ffi::iplSimulatorRunReflections(self.inner);
        }
    }

    /// Runs a pathing simulation for all sources added to the simulator.
    ///
    /// This function can be CPU intensive, and should be called from a separate
    /// thread in order to not block either the audio processing thread or
    /// the game's main update thread.
    pub fn run_pathing(&self) {
        unsafe {
            ffi::iplSimulatorRunPathing(self.inner);
        }
    }

    /// Creates a simulation source.
    pub fn create_source(&self) -> Result<Source> {
        let mut source_settings = ffi::IPLSourceSettings { flags: 0 };
        let mut source = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplSourceCreate(self.inner, &mut source_settings, &mut source),
                Source {
                    inner: source,
                    inputs: RefCell::new(std::mem::zeroed()),
                    simulator: self.clone(),
                },
            )
        }
    }
}

impl Clone for Simulator {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplSimulatorRetain(self.inner);
        }

        Self {
            inner: self.inner,
            shared_inputs: self.shared_inputs.clone(),
        }
    }
}

impl Drop for Simulator {
    fn drop(&mut self) {
        unsafe {
            ffi::iplSimulatorRelease(&mut self.inner);
        }
    }
}

unsafe impl Send for Simulator {}

unsafe impl Sync for Simulator {}

/// A sound source, for the purposes of simulation. This object is used to
/// specify various parameters for direct and indirect sound propagation
/// simulation, and to retrieve the simulation results.
pub struct Source {
    pub(crate) inner: ffi::IPLSource,
    pub(crate) inputs: RefCell<ffi::IPLSimulationInputs>,

    simulator: Simulator,
}

impl Source {
    /// Adds or removes a source to the set of sources processed by a simulator
    /// in subsequent simulations.
    pub fn set_active(&mut self, active: bool) {
        unsafe {
            if active {
                ffi::iplSourceAdd(self.inner, self.simulator.inner)
            } else {
                ffi::iplSourceRemove(self.inner, self.simulator.inner)
            }
        }
    }

    /// The position and orientation of this source.
    pub fn set_source(&mut self, source: Orientation) {
        self.inputs.get_mut().source = source.into();

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT
                    | ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_REFLECTIONS
                    | ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_PATHING,
                self.inputs.as_ptr(),
            );
        }
    }

    /// Apply frequency-independent distance attenuation.
    pub fn set_distance_attenuation(
        &mut self,
        distance_attenuation_model: DistanceAttenuationModel,
    ) {
        let inputs = self.inputs.get_mut();
        inputs.flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        inputs.directFlags |=
            ffi::IPLDirectSimulationFlags_IPL_DIRECTSIMULATIONFLAGS_DISTANCEATTENUATION;
        inputs.distanceAttenuationModel = distance_attenuation_model.into();

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }

    /// Apply frequency-dependent air absorption as a function of distance.
    pub fn set_air_absorption(&mut self, air_absorption_model: AirAbsorptionModel) {
        let inputs = self.inputs.get_mut();
        inputs.flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        inputs.directFlags |= ffi::IPLDirectSimulationFlags_IPL_DIRECTSIMULATIONFLAGS_AIRABSORPTION;
        inputs.airAbsorptionModel = air_absorption_model.into();

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }

    /// Apply attenuation due to source directivity pattern.
    pub fn set_directivity(&mut self, directivity: Directivity) {
        let inputs = self.inputs.get_mut();
        inputs.flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        inputs.directFlags |= ffi::IPLDirectSimulationFlags_IPL_DIRECTSIMULATIONFLAGS_DIRECTIVITY;
        inputs.directivity = directivity.into();

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }

    /// Apply occlusion.
    pub fn set_occlusion(&mut self) {
        let inputs = self.inputs.get_mut();
        inputs.flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        inputs.directFlags |= ffi::IPLDirectSimulationFlags_IPL_DIRECTSIMULATIONFLAGS_OCCLUSION;
        inputs.occlusionType = ffi::IPLOcclusionType_IPL_OCCLUSIONTYPE_RAYCAST;

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }

    /// Apply transmission along with occlusion.
    pub fn set_transmission(&mut self) {
        let inputs = self.inputs.get_mut();
        inputs.flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        inputs.directFlags |= ffi::IPLDirectSimulationFlags_IPL_DIRECTSIMULATIONFLAGS_TRANSMISSION;

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }

    pub fn set_reflections(&mut self) {
        self.inputs.get_mut().flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_REFLECTIONS;

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }
}

impl Clone for Source {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplSourceRetain(self.inner);
        }

        Self {
            inner: self.inner,
            inputs: self.inputs.clone(),
            simulator: self.simulator.clone(),
        }
    }
}

impl Drop for Source {
    fn drop(&mut self) {
        unsafe {
            ffi::iplSourceRelease(&mut self.inner);
        }
    }
}

unsafe impl Send for Source {}

unsafe impl Sync for Source {}

/// A distance attenuation model that can be used for modeling attenuation of
/// sound over distance. Can be used with both direct and indirect sound
/// propagation.
#[derive(Default)]
pub enum DistanceAttenuationModel {
    #[default]
    Default,
    InverseDistance(f32),
    Custom(Box<dyn Fn(f32) -> f32>),
}

impl From<DistanceAttenuationModel> for ffi::IPLDistanceAttenuationModel {
    fn from(value: DistanceAttenuationModel) -> Self {
        unsafe extern "C" fn callback_trampoline(
            distance: ffi::IPLfloat32,
            user_data: *mut std::os::raw::c_void,
        ) -> ffi::IPLfloat32 {
            let callback: &mut Box<dyn Fn(f32) -> f32> = unsafe { std::mem::transmute(user_data) };
            callback(distance)
        }

        match value {
            DistanceAttenuationModel::Default => Self {
                type_: ffi::IPLDistanceAttenuationModelType_IPL_DISTANCEATTENUATIONTYPE_DEFAULT,
                minDistance: 0.0,
                callback: None,
                userData: std::ptr::null_mut(),
                dirty: ffi::IPLbool_IPL_FALSE,
            },
            DistanceAttenuationModel::InverseDistance(distance) => Self {
                type_:
                    ffi::IPLDistanceAttenuationModelType_IPL_DISTANCEATTENUATIONTYPE_INVERSEDISTANCE,
                minDistance: distance,
                callback: None,
                userData: std::ptr::null_mut(),
                dirty: ffi::IPLbool_IPL_FALSE,
            },
            DistanceAttenuationModel::Custom(callback) => Self {
                type_: ffi::IPLDistanceAttenuationModelType_IPL_DISTANCEATTENUATIONTYPE_CALLBACK,
                minDistance: 0.0,
                callback: Some(callback_trampoline),
                userData: Box::into_raw(Box::new(callback)) as *mut _,
                dirty: ffi::IPLbool_IPL_FALSE,
            },
        }
    }
}

/// An air absorption model that can be used for modeling frequency-dependent
/// attenuation of sound over distance.
#[derive(Default)]
pub enum AirAbsorptionModel {
    #[default]
    Default,
    Exponential([f32; 3]),
    Custom(Box<dyn Fn(f32, u8) -> f32>),
}

impl From<AirAbsorptionModel> for ffi::IPLAirAbsorptionModel {
    fn from(value: AirAbsorptionModel) -> Self {
        unsafe extern "C" fn callback_trampoline(
            distance: ffi::IPLfloat32,
            band: ffi::IPLint32,
            user_data: *mut std::os::raw::c_void,
        ) -> ffi::IPLfloat32 {
            let callback: &mut Box<dyn Fn(f32, u8) -> f32> =
                unsafe { std::mem::transmute(user_data) };
            callback(distance, band as u8)
        }

        match value {
            AirAbsorptionModel::Default => Self {
                type_: ffi::IPLAirAbsorptionModelType_IPL_AIRABSORPTIONTYPE_DEFAULT,
                coefficients: [0.0, 0.0, 0.0],
                callback: None,
                userData: std::ptr::null_mut(),
                dirty: ffi::IPLbool_IPL_FALSE,
            },
            AirAbsorptionModel::Exponential(coefficients) => Self {
                type_: ffi::IPLAirAbsorptionModelType_IPL_AIRABSORPTIONTYPE_EXPONENTIAL,
                coefficients,
                callback: None,
                userData: std::ptr::null_mut(),
                dirty: ffi::IPLbool_IPL_FALSE,
            },
            AirAbsorptionModel::Custom(callback) => Self {
                type_: ffi::IPLDistanceAttenuationModelType_IPL_DISTANCEATTENUATIONTYPE_CALLBACK,
                coefficients: [0.0, 0.0, 0.0],
                callback: Some(callback_trampoline),
                userData: Box::into_raw(Box::new(callback)) as *mut _,
                dirty: ffi::IPLbool_IPL_FALSE,
            },
        }
    }
}

/// A directivity pattern that can be used to model changes in sound intensity
/// as a function of the source's orientation. Can be used with both direct and
/// indirect sound propagation.
///
/// The default directivity model is a weighted dipole. This is a linear blend
/// between an omnidirectional source (which emits sound with equal intensity in
/// all directions), and a dipole oriented along the z-axis in the source's
/// coordinate system (which focuses sound along the +z and -z axes). A callback
/// function can be specified to implement any other arbitrary directivity
/// pattern.
pub enum Directivity {
    Dipole { weight: f32, power: f32 },
    Custom(Box<dyn Fn(Vec3) -> f32>),
}

impl From<Directivity> for ffi::IPLDirectivity {
    fn from(value: Directivity) -> Self {
        unsafe extern "C" fn callback_trampoline(
            direction: ffi::IPLVector3,
            user_data: *mut std::os::raw::c_void,
        ) -> ffi::IPLfloat32 {
            let callback: &mut Box<dyn Fn(Vec3) -> f32> = unsafe { std::mem::transmute(user_data) };
            callback(direction.into())
        }

        match value {
            Directivity::Dipole { weight, power } => Self {
                dipoleWeight: weight,
                dipolePower: power,
                callback: None,
                userData: std::ptr::null_mut(),
            },
            Directivity::Custom(callback) => Self {
                dipoleWeight: 0.0,
                dipolePower: 0.0,
                callback: Some(callback_trampoline),
                userData: Box::into_raw(Box::new(callback)) as *mut _,
            },
        }
    }
}
