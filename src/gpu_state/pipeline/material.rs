use bytemuck::{Pod, Zeroable};
use rand::random;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MaterialStorage {
    color: [f32; 3],
    kind: u32,
    fuzz: f32,
    _padding: [f32; 3],
}

impl MaterialStorage {
    pub fn new(color: [f32; 3], kind: u32, fuzz: f32) -> MaterialStorage {
        MaterialStorage {
            color,
            kind,
            fuzz,

            _padding: [0.0, 0.0, 0.0],
        }
    }
}

