use super::material::MaterialStorage;
use bytemuck::{Pod, Zeroable};

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
    fn new(
        center: [f32; 3],
        radius: f32,
        u: [f32; 3],
        v: [f32; 3],
        kind: u32,
        material: MaterialStorage,
    ) -> GeometryStorage {
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

    /// `corner` is a position vector of one corner of the parallelogram.
    /// `u` and `v` are length vectors for the two arms of the parallelogram.
    pub fn new_quad(
        corner: [f32; 3],
        u: [f32; 3],
        v: [f32; 3],
        material: MaterialStorage,
    ) -> GeometryStorage {
        GeometryStorage::new(corner, 0.0, u, v, 1, material)
    }
}
