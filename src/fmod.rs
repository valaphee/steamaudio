use crate::{context::Context, ffi, ffi::IPLSimulationSettings, hrtf::Hrtf, simulation::Source};

pub fn init_fmod(context: &Context) {
    unsafe {
        ffi::iplFMODInitialize(context.inner);
    }
}

pub fn fmod_set_hrtf(hrtf: &Hrtf) {
    unsafe {
        ffi::iplFMODSetHRTF(hrtf.inner);
    }
}

pub fn fmod_add_source(source: &Source) -> i32 {
    unsafe { ffi::iplFMODAddSource(source.inner) }
}

pub fn fmod_set_simulation_settings(settings: IPLSimulationSettings) {
    unsafe {
        ffi::iplFMODSetSimulationSettings(settings);
    }
}

//todo: Better API for this
pub fn fmod_create_settings(sampling_rate: u32, frame_size: u32) -> IPLSimulationSettings {
    IPLSimulationSettings {
        flags: ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
        sceneType: ffi::IPLSceneType_IPL_SCENETYPE_DEFAULT,
        reflectionType: 0,
        maxNumOcclusionSamples: 8,
        maxNumRays: 32,
        numDiffuseSamples: 8,
        maxDuration: 1.0,
        maxOrder: 1,
        maxNumSources: 256,
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
