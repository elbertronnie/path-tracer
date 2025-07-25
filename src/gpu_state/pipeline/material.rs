use bytemuck::{Pod, Zeroable};

pub enum MaterialKind {
    Lambertian,
    Metallic { fuzz: f32 },
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MaterialStorage {
    color: [f32; 3],
    kind: u32,
    fuzz: f32,
    _padding: [f32; 3],
}

impl MaterialStorage {
    fn new_with_padding(color: [f32; 3], kind: u32, fuzz: f32) -> MaterialStorage {
        MaterialStorage {
            color,
            kind,
            fuzz,

            _padding: [0.0, 0.0, 0.0],
        }
    }

    pub fn new(color: [f32; 3], kind: MaterialKind) -> MaterialStorage {
        match kind {
            MaterialKind::Lambertian => MaterialStorage::new_with_padding(color, 0, 0.0),
            MaterialKind::Metallic { fuzz } => MaterialStorage::new_with_padding(color, 1, fuzz),
        }
    }

    pub fn new_lambertian(color: [f32; 3]) -> MaterialStorage {
        MaterialStorage::new(color, MaterialKind::Lambertian)
    }

    pub fn new_metallic(color: [f32; 3], fuzz: f32) -> MaterialStorage {
        MaterialStorage::new(color, MaterialKind::Metallic { fuzz })
    }
}
