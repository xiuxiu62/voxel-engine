use crate::voxel::Voxel;
use bevy::{ecs::component::Component, math::Vec3};

#[derive(Debug, Component)]
pub struct Chunk {
    voxels: Vec<Voxel>,
    pub position: Vec3,
}

impl Chunk {
    pub const SIZE: usize = 16;

    #[inline]
    pub fn new(position: Vec3) -> Self {
        Self {
            voxels: vec![Voxel { id: 0 }; Self::SIZE * Self::SIZE * Self::SIZE],
            position,
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&Voxel> {
        self.voxels.get(Self::linearize(x, y, z))
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: Voxel) {
        if x < Self::SIZE && y < Self::SIZE && z < Self::SIZE {
            let i = Self::linearize(x, y, z);
            self.voxels[i] = value;
        }
    }

    #[inline]
    const fn linearize(x: usize, y: usize, z: usize) -> usize {
        (z * Self::SIZE * Self::SIZE) + (y * Self::SIZE) + x
    }
}
