use glam::Mat4;

use crate::{
    context::Context,
    error::{check, Result},
    ffi,
};

impl Context {
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
}

/// A 3D scene, which can contain geometry objects that can interact with
/// acoustic rays. The scene object itself doesn't contain any geometry, but is
/// a container for \c IPLStaticMesh and \c IPLInstancedMesh objects, which
/// do contain geometry.
pub struct Scene {
    pub(crate) inner: ffi::IPLScene,

    context: Context,
}

impl Scene {
    pub fn create_static_mesh(
        &self,
        indices: &[[u32; 3]],
        positions: &[[f32; 3]],
        material_indices: &[u32],
        materials: &[Material],
    ) -> Result<StaticMesh> {
        unsafe {
            let mut static_mesh_settings = ffi::IPLStaticMeshSettings {
                numVertices: positions.len() as i32,
                numTriangles: indices.len() as i32,
                numMaterials: materials.len() as i32,
                vertices: std::mem::transmute(positions.as_ptr()),
                triangles: std::mem::transmute(indices.as_ptr()),
                materialIndices: std::mem::transmute(material_indices.as_ptr()),
                materials: std::mem::transmute(materials.as_ptr()),
            };
            let mut static_mesh = std::ptr::null_mut();

            check(
                ffi::iplStaticMeshCreate(self.inner, &mut static_mesh_settings, &mut static_mesh),
                StaticMesh {
                    inner: static_mesh,
                    scene: self.clone(),
                },
            )
        }
    }

    pub fn create_instanced_mesh(&self, scene: &Scene, transform: Mat4) -> Result<InstancedMesh> {
        let mut instanced_mesh_settings = ffi::IPLInstancedMeshSettings {
            subScene: scene.inner,
            transform: transform.into(),
        };
        let mut instanced_mesh = std::ptr::null_mut();

        unsafe {
            check(
                ffi::iplInstancedMeshCreate(
                    self.inner,
                    &mut instanced_mesh_settings,
                    &mut instanced_mesh,
                ),
                InstancedMesh {
                    inner: instanced_mesh,
                    scene: self.clone(),
                    sub_scene: scene.clone(),
                },
            )
        }
    }

    /// Commits any changes to the scene.
    pub fn commit(&self) {
        unsafe {
            ffi::iplSceneCommit(self.inner);
        }
    }
}

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

unsafe impl Send for Scene {}

unsafe impl Sync for Scene {}

/// A triangle mesh that doesn't move or deform in any way. The unchanging
/// portions of a scene should typically be collected into a single static mesh
/// object. In addition to the geometry, a static mesh also contains
/// acoustic material information for each triangle.
pub struct StaticMesh {
    inner: ffi::IPLStaticMesh,

    scene: Scene,
}

impl StaticMesh {
    /// Add or removes a static mesh from a scene.
    pub fn set_visible(&mut self, visible: bool) {
        unsafe {
            if visible {
                ffi::iplStaticMeshAdd(self.inner, self.scene.inner)
            } else {
                ffi::iplStaticMeshRemove(self.inner, self.scene.inner)
            }
        }
    }
}

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
        unsafe {
            ffi::iplStaticMeshRelease(&mut self.inner);
        }
    }
}

unsafe impl Send for StaticMesh {}

unsafe impl Sync for StaticMesh {}

/// A triangle mesh that can be moved (translated), rotated, or scaled, but
/// cannot deform. Portions of a scene that undergo rigid-body motion can be
/// represented as instanced meshes. An instanced mesh is essentially a
/// scene (called the "sub-scene") with a transform applied to it. Adding an
/// instanced mesh to a scene places the sub-scene into the scene with the
/// transform applied. For example, the sub-scene may be a prefab door,
/// and the transform can be used to place it in a doorway and animate it as it
/// opens or closes.
pub struct InstancedMesh {
    inner: ffi::IPLInstancedMesh,

    scene: Scene,
    sub_scene: Scene,
}

impl InstancedMesh {
    /// Add or removes an instanced mesh from a scene.
    pub fn set_visible(&mut self, visible: bool) {
        unsafe {
            if visible {
                ffi::iplInstancedMeshAdd(self.inner, self.scene.inner)
            } else {
                ffi::iplInstancedMeshRemove(self.inner, self.scene.inner)
            }
        }
    }

    /// Updates the local-to-world transform of an instanced mesh within its
    /// parent scene.
    ///
    /// This function allows the instanced mesh to be moved, rotated, and scaled
    /// dynamically.
    pub fn set_transform(&mut self, transform: Mat4) {
        unsafe {
            ffi::iplInstancedMeshUpdateTransform(self.inner, self.scene.inner, transform.into());
        }
    }
}

impl Clone for InstancedMesh {
    fn clone(&self) -> Self {
        unsafe {
            ffi::iplInstancedMeshRetain(self.inner);
        }

        Self {
            inner: self.inner,
            scene: self.scene.clone(),
            sub_scene: self.sub_scene.clone(),
        }
    }
}

impl Drop for InstancedMesh {
    fn drop(&mut self) {
        unsafe {
            ffi::iplInstancedMeshRelease(&mut self.inner);
        }
    }
}

unsafe impl Send for InstancedMesh {}

unsafe impl Sync for InstancedMesh {}

/// The acoustic properties of a surface.
///
/// You can specify the acoustic material properties of each triangle, although
/// typically many triangles will share a common material.
///
/// The acoustic material properties are specified for three frequency bands
/// with center frequencies of 400 Hz, 2.5 KHz, and 15 KHz.
///
/// Below are the acoustic material properties for a few standard materials.
///
/// {"generic",{0.10f,0.20f,0.30f,0.05f,0.100f,0.050f,0.030f}}
/// {"brick",{0.03f,0.04f,0.07f,0.05f,0.015f,0.015f,0.015f}}
/// {"concrete",{0.05f,0.07f,0.08f,0.05f,0.015f,0.002f,0.001f}}
/// {"ceramic",{0.01f,0.02f,0.02f,0.05f,0.060f,0.044f,0.011f}}
/// {"gravel",{0.60f,0.70f,0.80f,0.05f,0.031f,0.012f,0.008f}},
/// {"carpet",{0.24f,0.69f,0.73f,0.05f,0.020f,0.005f,0.003f}}
/// {"glass",{0.06f,0.03f,0.02f,0.05f,0.060f,0.044f,0.011f}}
/// {"plaster",{0.12f,0.06f,0.04f,0.05f,0.056f,0.056f,0.004f}}
/// {"wood",{0.11f,0.07f,0.06f,0.05f,0.070f,0.014f,0.005f}}
/// {"metal",{0.20f,0.07f,0.06f,0.05f,0.200f,0.025f,0.010f}}
/// {"rock",{0.13f,0.20f,0.24f,0.05f,0.015f,0.002f,0.001f}}
#[repr(C)]
pub struct Material {
    /// Fraction of sound energy absorbed at low, middle, high frequencies.
    /// Between 0.0 and 1.0.
    pub absorption: [f32; 3],

    /// Fraction of sound energy scattered in a random direction on reflection.
    /// Between 0.0 (pure specular) and 1.0 (pure diffuse).
    pub scattering: f32,

    /// Fraction of sound energy transmitted through at low, middle, high
    /// frequencies. Between 0.0 and 1.0. Only used for direct occlusion
    /// calculations.
    pub transmission: [f32; 3],
}
