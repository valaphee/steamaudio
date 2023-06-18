use std::cell::RefCell;

use glam::Vec3;

use crate::{
    buffer::SpeakerLayout,
    effect::*,
    error::{check, Result},
    ffi,
    geometry::Orientation,
    hrtf::Hrtf,
    scene::Scene,
    simulation::Simulator,
};

/// A context object, which controls low-level operations of Steam Audio.
/// Typically, a context is specified once during the execution of the client
/// program, before calling any other API functions
pub struct Context {
    pub(crate) inner: ffi::IPLContext,
}

impl Context {
    /// Creates a context object. A context must be created before creating any
    /// other API objects.
    pub fn new() -> Result<Self> {
        unsafe extern "C" fn log_callback(
            level: ffi::IPLLogLevel,
            message: *const std::os::raw::c_char,
        ) {
            let message = std::ffi::CStr::from_ptr(message).to_str().unwrap();
            match level {
                ffi::IPLLogLevel_IPL_LOGLEVEL_INFO => {
                    tracing::info!(message);
                }
                ffi::IPLLogLevel_IPL_LOGLEVEL_WARNING => {
                    tracing::warn!(message);
                }
                ffi::IPLLogLevel_IPL_LOGLEVEL_ERROR => {
                    tracing::error!(message);
                }
                ffi::IPLLogLevel_IPL_LOGLEVEL_DEBUG => {
                    tracing::debug!(message);
                }
                _ => unreachable!(),
            }
        }

        struct AllocInfo {
            layout: std::alloc::Layout,
            ptr: *mut u8,
        }

        unsafe extern "C" fn allocate_callback(
            size: ffi::IPLsize,
            alignment: ffi::IPLsize,
        ) -> *mut std::ffi::c_void {
            std::alloc::Layout::from_size_align(size, alignment).map_or_else(
                |_| std::ptr::null_mut(),
                |layout| {
                    let alloc_info_layout = std::alloc::Layout::new::<AllocInfo>();
                    let (alloc_layout, offset) = alloc_info_layout.extend(layout).unwrap();

                    let alloc_ptr = std::alloc::alloc(alloc_layout);
                    if alloc_ptr.is_null() {
                        return alloc_ptr;
                    }

                    let ptr = alloc_ptr.add(offset);
                    let alloc_info_ptr =
                        ptr.sub(std::mem::size_of::<AllocInfo>()) as *mut AllocInfo;
                    alloc_info_ptr.write_unaligned(AllocInfo {
                        layout: alloc_layout,
                        ptr: alloc_ptr,
                    });

                    ptr
                },
            ) as *mut std::ffi::c_void
        }

        unsafe extern "C" fn free_callback(ptr: *mut std::ffi::c_void) {
            assert!(!ptr.is_null());

            let alloc_info_ptr = ptr.sub(std::mem::size_of::<AllocInfo>()) as *const AllocInfo;
            let alloc_info = alloc_info_ptr.read_unaligned();
            std::alloc::dealloc(alloc_info.ptr, alloc_info.layout);
        }

        let mut context_settings = ffi::IPLContextSettings {
            version: ffi::STEAMAUDIO_VERSION_MAJOR << 16
                | ffi::STEAMAUDIO_VERSION_MINOR << 8
                | ffi::STEAMAUDIO_VERSION_PATCH,
            logCallback: Some(log_callback),
            allocateCallback: Some(allocate_callback),
            freeCallback: Some(free_callback),
            simdLevel: ffi::IPLSIMDLevel_IPL_SIMDLEVEL_AVX512,
        };
        let mut context = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplContextCreate(&mut context_settings, &mut context),
                Self { inner: context },
            )
        }
    }

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

    /// Creates a scene.
    ///
    /// A scene does not store any geometry information on its own; for that you
    /// need to create one or more static meshes or instanced meshes and add
    /// them to the scene.
    pub fn create_scene(&self) -> Result<Scene> {
        let mut scene_settings = ffi::IPLSceneSettings {
            type_: ffi::IPLSceneType_IPL_SCENETYPE_DEFAULT,
            closestHitCallback: None,
            anyHitCallback: None,
            batchedClosestHitCallback: None,
            batchedAnyHitCallback: None,
            userData: std::ptr::null_mut(),
            embreeDevice: std::ptr::null_mut(),
            radeonRaysDevice: std::ptr::null_mut(),
        };
        let mut scene = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplSceneCreate(self.inner, &mut scene_settings, &mut scene),
                Scene {
                    context: self.clone(),
                    inner: scene,
                },
            )
        }
    }

    /// Creates an HRTF.
    ///
    /// Calling this function is somewhat expensive; avoid creating HRTF objects
    /// in your audio thread at all if possible.
    ///
    /// This function is not thread-safe. Do not simultaneously call it from
    /// multiple threads.
    pub fn create_hrtf(&self, sampling_rate: u32, frame_size: u32) -> Result<Hrtf> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut hrtf_settings = ffi::IPLHRTFSettings {
            type_: ffi::IPLHRTFType_IPL_HRTFTYPE_DEFAULT,
            sofaFileName: std::ptr::null_mut(),
            sofaData: std::ptr::null_mut(),
            sofaDataSize: 0,
            volume: 1.0,
        };
        let mut hrtf = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplHRTFCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut hrtf_settings,
                    &mut hrtf,
                ),
                Hrtf {
                    inner: hrtf,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates a panning effect.
    pub fn create_panning_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        speaker_layout: SpeakerLayout,
    ) -> Result<PanningEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut panning_effect_settings = ffi::IPLPanningEffectSettings {
            speakerLayout: speaker_layout.into(),
        };
        let mut panning_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplPanningEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut panning_effect_settings,
                    &mut panning_effect,
                ),
                PanningEffect {
                    inner: panning_effect,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates a binaural effect.
    pub fn create_binaural_effect(
        &self,
        hrtf: &Hrtf,
        sampling_rate: u32,
        frame_size: u32,
    ) -> Result<BinauralEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut binaural_effect_settings = ffi::IPLBinauralEffectSettings { hrtf: hrtf.inner };
        let mut binaural_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplBinauralEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut binaural_effect_settings,
                    &mut binaural_effect,
                ),
                BinauralEffect {
                    inner: binaural_effect,
                    context: self.clone(),
                    hrtf: hrtf.clone(),
                },
            )
        }
    }

    /// Creates a virtual surround effect.
    pub fn create_virtual_surround_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        speaker_layout: SpeakerLayout,
        hrtf: &Hrtf,
    ) -> Result<VirtualSurroundEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut virtual_surround_effect_settings = ffi::IPLVirtualSurroundEffectSettings {
            speakerLayout: speaker_layout.into(),
            hrtf: hrtf.inner,
        };
        let mut virtual_surround_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplVirtualSurroundEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut virtual_surround_effect_settings,
                    &mut virtual_surround_effect,
                ),
                VirtualSurroundEffect {
                    inner: virtual_surround_effect,
                    context: self.clone(),
                    hrtf: hrtf.clone(),
                },
            )
        }
    }

    /// Creates an Ambisonics encode effect.
    pub fn create_ambisonics_encode_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        maximum_order: u8,
    ) -> Result<AmbisonicsEncodeEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut ambisonics_encode_effect_settings = ffi::IPLAmbisonicsEncodeEffectSettings {
            maxOrder: maximum_order as i32,
        };
        let mut ambisonics_encode_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsEncodeEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut ambisonics_encode_effect_settings,
                    &mut ambisonics_encode_effect,
                ),
                AmbisonicsEncodeEffect {
                    inner: ambisonics_encode_effect,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates an Ambisonics panning effect.
    pub fn create_ambisonics_panning_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        speaker_layout: SpeakerLayout,
        maximum_order: u8,
    ) -> Result<AmbisonicsPanningEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut ambisonics_panning_effect_settings = ffi::IPLAmbisonicsPanningEffectSettings {
            speakerLayout: speaker_layout.into(),
            maxOrder: maximum_order as i32,
        };
        let mut ambisonics_panning_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsPanningEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut ambisonics_panning_effect_settings,
                    &mut ambisonics_panning_effect,
                ),
                AmbisonicsPanningEffect {
                    inner: ambisonics_panning_effect,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates an Ambisonics binaural effect.
    pub fn create_ambisonics_binaural_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        hrtf: &Hrtf,
        maximum_order: u8,
    ) -> Result<AmbisonicsBinauralEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut ambisonics_binaural_effect_settings = ffi::IPLAmbisonicsBinauralEffectSettings {
            hrtf: hrtf.inner,
            maxOrder: maximum_order as i32,
        };
        let mut ambisonics_binaural_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsBinauralEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut ambisonics_binaural_effect_settings,
                    &mut ambisonics_binaural_effect,
                ),
                AmbisonicsBinauralEffect {
                    inner: ambisonics_binaural_effect,
                    context: self.clone(),
                    hrtf: hrtf.clone(),
                },
            )
        }
    }

    /// Creates an Ambisonics rotation effect.
    pub fn create_ambisonics_rotation_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        maximum_order: u8,
    ) -> Result<AmbisonicsRotationEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut ambisonics_rotation_effect_settings = ffi::IPLAmbisonicsRotationEffectSettings {
            maxOrder: maximum_order as i32,
        };
        let mut ambisonics_rotation_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsRotationEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut ambisonics_rotation_effect_settings,
                    &mut ambisonics_rotation_effect,
                ),
                AmbisonicsRotationEffect {
                    inner: ambisonics_rotation_effect,
                    context: self.clone(),
                },
            )
        }
    }

    /// Creates an Ambisonics decode effect.
    pub fn create_ambisonics_decode_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        speaker_layout: SpeakerLayout,
        hrtf: &Hrtf,
        maximum_order: u8,
    ) -> Result<AmbisonicsDecodeEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut ambisonics_decode_effect_settings = ffi::IPLAmbisonicsDecodeEffectSettings {
            speakerLayout: speaker_layout.into(),
            hrtf: hrtf.inner,
            maxOrder: maximum_order as i32,
        };
        let mut ambisonics_decode_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplAmbisonicsDecodeEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut ambisonics_decode_effect_settings,
                    &mut ambisonics_decode_effect,
                ),
                AmbisonicsDecodeEffect {
                    inner: ambisonics_decode_effect,
                    context: self.clone(),
                    hrtf: hrtf.clone(),
                },
            )
        }
    }

    pub fn create_direct_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        channels: u16,
    ) -> Result<DirectEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut direct_effect_settings = ffi::IPLDirectEffectSettings {
            numChannels: channels as i32,
        };
        let mut direct_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplDirectEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut direct_effect_settings,
                    &mut direct_effect,
                ),
                DirectEffect {
                    inner: direct_effect,
                    context: self.clone(),
                },
            )
        }
    }

    pub fn create_reflection_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        channels: u16,
    ) -> Result<ReflectionEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut reflection_effect_settings = ffi::IPLReflectionEffectSettings {
            type_: ffi::IPLReflectionEffectType_IPL_REFLECTIONEFFECTTYPE_CONVOLUTION,
            irSize: (2 * sampling_rate) as i32,
            numChannels: channels as i32,
        };
        let mut reflection_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplReflectionEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut reflection_effect_settings,
                    &mut reflection_effect,
                ),
                ReflectionEffect {
                    inner: reflection_effect,
                    context: self.clone(),
                },
            )
        }
    }

    pub fn create_path_effect(
        &self,
        sampling_rate: u32,
        frame_size: u32,
        maximum_order: u8,
    ) -> Result<PathEffect> {
        let mut audio_settings = ffi::IPLAudioSettings {
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
        };
        let mut path_effect_settings = ffi::IPLPathEffectSettings {
            maxOrder: maximum_order as i32,
        };
        let mut path_effect = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplPathEffectCreate(
                    self.inner,
                    &mut audio_settings,
                    &mut path_effect_settings,
                    &mut path_effect,
                ),
                PathEffect {
                    inner: path_effect,
                    context: self.clone(),
                },
            )
        }
    }

    pub fn create_simulator(&self, sampling_rate: u32, frame_size: u32) -> Result<Simulator> {
        let mut simulation_settings = ffi::IPLSimulationSettings {
            flags: ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
            sceneType: ffi::IPLSceneType_IPL_SCENETYPE_DEFAULT,
            reflectionType: 0,
            maxNumOcclusionSamples: 0,
            maxNumRays: 0,
            numDiffuseSamples: 0,
            maxDuration: 0.0,
            maxOrder: 0,
            maxNumSources: 0,
            numThreads: 0,
            rayBatchSize: 0,
            numVisSamples: 0,
            samplingRate: sampling_rate as i32,
            frameSize: frame_size as i32,
            openCLDevice: std::ptr::null_mut(),
            radeonRaysDevice: std::ptr::null_mut(),
            tanDevice: std::ptr::null_mut(),
        };
        let mut simulator = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplSimulatorCreate(self.inner, &mut simulation_settings, &mut simulator),
                Simulator {
                    inner: simulator,
                    shared_inputs: RefCell::new(std::mem::zeroed()),
                },
            )
        }
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplContextRetain(self.inner);
        }

        Self { inner: self.inner }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            ffi::iplContextRelease(&mut self.inner);
        }
    }
}

unsafe impl Send for Context {}

unsafe impl Sync for Context {}
