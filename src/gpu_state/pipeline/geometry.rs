use bytemuck::{Pod, Zeroable};
use rand::random;
use super::material::MaterialStorage;


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct GeometryStorage {
    center: [f32; 3],
    radius: f32,
    u: [f32; 3],
    _padding: f32,
    v: [f32; 3],
    kind: u32,
    material: MaterialStorage,
}

impl GeometryStorage {
    pub fn new(center: [f32; 3], radius: f32, u: [f32; 3], v: [f32; 3], kind: u32, material: MaterialStorage) -> GeometryStorage {
        GeometryStorage {
            center,
            radius,
            u,
            v,
            kind,
            material,

            _padding: 0.0,
        }
    }

    pub fn new_sphere(center: [f32; 3], radius: f32, material: MaterialStorage) -> GeometryStorage {
        GeometryStorage::new(center, radius, [0.0; 3], [0.0; 3], 0, material)
    }

    pub fn new_quad(center: [f32; 3], u: [f32; 3], v: [f32; 3], material: MaterialStorage) -> GeometryStorage {
        GeometryStorage::new(center, 0.0, u, v, 1, material)
    }
}
