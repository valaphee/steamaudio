use std::rc::Rc;

use glam::{Quat, Vec3};

use crate::error::check;
use crate::ffi;
use crate::prelude::*;

pub struct Simulator {
    pub(crate) inner: ffi::IPLSimulator,

    pub context: Context,
}

impl Simulator {
    pub fn new(context: &Context, sample_rate: u32, frame_length: u32, scene: &Scene) -> Result<Simulator, Error> {
        let mut settings: ffi::IPLSimulationSettings = unsafe { std::mem::zeroed() };
        settings.flags = ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        settings.samplingRate = sample_rate as i32;
        settings.frameSize = frame_length as i32;
        let mut simulator = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplSimulatorCreate(context.inner, &mut settings, &mut simulator),
                (),
            )?;
            ffi::iplSimulatorSetScene(simulator, scene.inner);
        }

        Ok(Self {
            inner: simulator,
            context: context.clone(),
        })
    }

    pub fn update(&mut self, listener: Orientation) {
        let mut shared_inputs: ffi::IPLSimulationSharedInputs = unsafe { std::mem::zeroed() };
        shared_inputs.listener = listener.into();
        shared_inputs.order = 2;

        unsafe {
            ffi::iplSimulatorSetSharedInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                &mut shared_inputs,
            )
        }
    }

    pub fn commit(&self) {
        unsafe {
            ffi::iplSimulatorCommit(self.inner);
        }
    }

    pub fn run_direct(&mut self) {
        unsafe { ffi::iplSimulatorRunDirect(self.inner) }
    }
}

unsafe impl Sync for Simulator {}

unsafe impl Send for Simulator {}

impl Clone for Simulator {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplSimulatorRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
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

pub struct Source {
    pub(crate) inner: ffi::IPLSource,

    pub simulator: Simulator,
}

impl Source {
    pub fn new(simulator: &Simulator) -> Result<Source, Error> {
        let mut settings = ffi::IPLSourceSettings {
            flags: ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
        };
        let mut source = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplSourceCreate(simulator.inner, &mut settings, &mut source),
                (),
            )?;
        }

        let mut source = Source {
            inner: source,
            simulator: simulator.clone(),
        };
        source.set_active(true);
        Ok(source)
    }

    pub fn set_active(&mut self, active: bool) {
        unsafe {
            match active {
                true => {
                    ffi::iplSourceAdd(self.inner, self.simulator.inner);
                }
                false => {
                    ffi::iplSourceRemove(self.inner, self.simulator.inner);
                }
            }
        }
    }

    pub fn update(&mut self, orientation: Orientation) {
        let mut inputs: ffi::IPLSimulationInputs = unsafe { std::mem::zeroed() };
        inputs.flags = ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT;
        inputs.directFlags = ffi::IPLDirectSimulationFlags_IPL_DIRECTSIMULATIONFLAGS_DISTANCEATTENUATION
            | ffi::IPLDirectSimulationFlags_IPL_DIRECTSIMULATIONFLAGS_AIRABSORPTION | ffi::IPLDirectSimulationFlags_IPL_DIRECTSIMULATIONFLAGS_DIRECTIVITY;
        inputs.source = orientation.into();

        unsafe {
            ffi::iplSourceSetInputs(
                self.inner,
                ffi::IPLSimulationFlags_IPL_SIMULATIONFLAGS_DIRECT,
                &mut inputs,
            )
        }
    }
}

unsafe impl Sync for Source {}
unsafe impl Send for Source {}

impl Clone for Source {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplSourceRetain(self.inner);
        }

        Self {
            inner: self.inner,
            simulator: self.simulator.clone(),
        }
    }
}

impl Drop for Source {
    fn drop(&mut self) {
        self.set_active(false);

        unsafe {
            ffi::iplSourceRelease(&mut self.inner);
        }
    }
}

pub struct Scene {
    pub(crate) inner: ffi::IPLScene,

    context: Context,
}

impl Scene {
    pub fn new(context: &Context) -> Result<Scene, Error> {
        let mut settings = ffi::IPLSceneSettings {
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
                ffi::iplSceneCreate(context.inner, &mut settings, &mut scene),
                (),
            )?;
        }

        Ok(Scene {
            inner: scene,
            context: context.clone(),
        })
    }

    pub fn commit(&mut self) {
        unsafe { ffi::iplSceneCommit(self.inner) }
    }
}

unsafe impl Sync for Scene {}
unsafe impl Send for Scene {}

impl Clone for Scene {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplSceneRetain(self.inner);
        }

        Self {
            inner: self.inner,
            context: self.context.clone(),
        }
    }
}

impl Drop for Scene {
    fn drop(&mut self) {
        unsafe {
            ffi::iplSceneRelease(&mut self.inner);
        }
    }
}

pub struct StaticMesh {
    pub(crate) inner: ffi::IPLStaticMesh,

    scene: Scene,
}

impl StaticMesh {
    pub fn new(
        scene: &Scene,
        vertices: &[[f32; 3]],
        triangles: &[[u32; 3]],
        material_indices: &[u32],
        materials: &[Material],
    ) -> Result<Self, Error> {
        let mut vertices = vertices
            .into_iter()
            .map(|vertex| ffi::IPLVector3 {
                x: vertex[0],
                y: vertex[1],
                z: vertex[2],
            })
            .collect::<Vec<_>>();
        let mut triangles = triangles
            .into_iter()
            .map(|triangle| ffi::IPLTriangle {
                indices: triangle.map(|index| index as i32),
            })
            .collect::<Vec<_>>();
        let mut material_indices = material_indices
            .iter()
            .map(|index| index.to_owned() as i32)
            .collect::<Vec<_>>();
        let mut materials = materials
            .into_iter()
            .map(|material| material.into())
            .collect::<Vec<_>>();
        let mut settings = ffi::IPLStaticMeshSettings {
            numVertices: vertices.len() as i32,
            numTriangles: triangles.len() as i32,
            numMaterials: materials.len() as i32,
            vertices: vertices.as_mut_ptr(),
            triangles: triangles.as_mut_ptr(),
            materialIndices: material_indices.as_mut_ptr(),
            materials: materials.as_mut_ptr(),
        };
        let mut static_mesh = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplStaticMeshCreate(scene.inner, &mut settings, &mut static_mesh),
                (),
            )?;
        }

        let mut static_mesh = StaticMesh {
            inner: static_mesh,
            scene: scene.clone(),
        };
        static_mesh.set_visible(true);

        Ok(static_mesh)
    }

    pub fn set_visible(&mut self, visible: bool) {
        unsafe {
            match visible {
                true => {
                    ffi::iplStaticMeshAdd(self.inner, self.scene.inner);
                }
                false => {
                    ffi::iplStaticMeshRemove(self.inner, self.scene.inner);
                }
            }
        }
    }
}

unsafe impl Sync for StaticMesh {}
unsafe impl Send for StaticMesh {}

impl Clone for StaticMesh {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplStaticMeshRetain(self.inner);
        }

        Self {
            inner: self.inner,
            scene: self.scene.clone(),
        }
    }
}

impl Drop for StaticMesh {
    fn drop(&mut self) {
        self.set_visible(false);

        unsafe {
            ffi::iplStaticMeshRelease(&mut self.inner);
        }
    }
}

pub struct Material {
    pub absorption: [f32; 3],
    pub scattering: f32,
    pub transmission: [f32; 3],
}

impl From<&Material> for ffi::IPLMaterial {
    fn from(value: &Material) -> Self {
        Self {
            absorption: value.absorption,
            scattering: value.scattering,
            transmission: value.transmission,
        }
    }
}

pub enum DistanceAttenuationModel<T>
where
    T: Fn(f32, f32) -> f32
{
    Exponential([f32; 3]),
    Callback(T)
}
