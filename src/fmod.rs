use crate::{context::Context, ffi, ffi::IPLSimulationSettings, hrtf::Hrtf, simulation::Source};

pub fn init_fmod(context: &Context) {
    unsafe {
        ffi::iplFMODInitialize(context.inner);
    }
}

pub fn terminate_fmod() {
    unsafe {
        ffi::iplFMODTerminate();
    }
}

pub fn set_hrtf(hrtf: &Hrtf) {
    unsafe {
        ffi::iplFMODSetHRTF(hrtf.inner);
    }
}

pub fn set_simulation_settings(settings: IPLSimulationSettings) {
    unsafe {
        ffi::iplFMODSetSimulationSettings(settings);
    }
}

pub fn set_reverb_source(source: &Source) {
    unsafe { ffi::iplFMODSetReverbSource(source.inner) }
}

pub fn add_source(source: &Source) -> i32 {
    unsafe { ffi::iplFMODAddSource(source.inner) }
}

pub fn remove_source(source_handle: i32) {
    unsafe { ffi::iplFMODRemoveSource(source_handle) }
}

//todo: Better API for this
pub fn fmod_create_settings(sampling_rate: u32, frame_size: u32) -> IPLSimulationSettings {
    IPLSimulationSettings {
        flags: ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT | ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_REFLECTIONS,
        sceneType: ffi::IPLSceneType_IPL_SCENETYPE_DEFAULT,
        reflectionType: ffi::IPLReflectionEffectType_IPL_REFLECTIONEFFECTTYPE_CONVOLUTION,
        maxNumOcclusionSamples: 8,
        maxNumRays: 4096,
        numDiffuseSamples: 32,
        maxDuration: 2.0,
        maxOrder: 1,
        maxNumSources: 8,
        numThreads: 2,
        rayBatchSize: 0,
        numVisSamples: 0,
        samplingRate: sampling_rate as i32,
        frameSize: frame_size as i32,
        openCLDevice: std::ptr::null_mut(),
        radeonRaysDevice: std::ptr::null_mut(),
        tanDevice: std::ptr::null_mut(),
    }
}
