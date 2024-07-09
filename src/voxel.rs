#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Voxel {
    pub id: u8,
}

impl Voxel {
    pub const SIZE: f32 = 1.0;
}
