use std::cell::RefCell;

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

    /// Specifies simulation parameters for a source.
    pub fn set_source(&mut self, source: Orientation) {
        self.inputs.get_mut().source = source.into();

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }

    pub fn set_distance_attenuation(&mut self) {
        self.inputs.get_mut().flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        self.inputs.get_mut().directFlags |=
            ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYDISTANCEATTENUATION;
        self.inputs.get_mut().distanceAttenuationModel = ffi::IPLDistanceAttenuationModel {
            type_: ffi::IPLDistanceAttenuationModelType_IPL_DISTANCEATTENUATIONTYPE_DEFAULT,
            minDistance: 0.0,
            callback: None,
            userData: std::ptr::null_mut(),
            dirty: ffi::IPLbool_IPL_FALSE,
        };

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }

    pub fn set_air_absorption(&mut self) {
        self.inputs.get_mut().flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        self.inputs.get_mut().directFlags |=
            ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYAIRABSORPTION;
        self.inputs.get_mut().airAbsorptionModel = ffi::IPLAirAbsorptionModel {
            type_: ffi::IPLAirAbsorptionModelType_IPL_AIRABSORPTIONTYPE_DEFAULT,
            coefficients: [0.0, 0.0, 0.0],
            callback: None,
            userData: std::ptr::null_mut(),
            dirty: ffi::IPLbool_IPL_FALSE,
        };

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }

    pub fn set_directivity(&mut self) {
        self.inputs.get_mut().flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        self.inputs.get_mut().directFlags |=
            ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYDIRECTIVITY;
        self.inputs.get_mut().directivity = ffi::IPLDirectivity {
            dipoleWeight: 0.0,
            dipolePower: 0.0,
            callback: None,
            userData: std::ptr::null_mut(),
        };

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }

    pub fn set_occlusion(&mut self) {
        self.inputs.get_mut().flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        self.inputs.get_mut().directFlags |=
            ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYOCCLUSION;
        self.inputs.get_mut().occlusionType = ffi::IPLOcclusionType_IPL_OCCLUSIONTYPE_RAYCAST;
        self.inputs.get_mut().occlusionRadius = 0.0;
        self.inputs.get_mut().numOcclusionSamples = 0;

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                self.inputs.as_ptr(),
            );
        }
    }

    pub fn set_transmission(&mut self) {
        self.inputs.get_mut().flags |= ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        self.inputs.get_mut().directFlags |=
            ffi::IPLDirectEffectFlags_IPL_DIRECTEFFECTFLAGS_APPLYTRANSMISSION;
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
