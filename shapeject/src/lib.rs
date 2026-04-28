use std::ops::{Deref, DerefMut};

use cgmath::Vector3;
use spatial_hash_3d::{BoxIdxIterator, BoxIterator, BoxIteratorMut, SpatialHashGrid};

pub struct SpatialHash3D<T> {
    cube_sidelength: f32,
    bottom_left: Vector3<f32>,
    grid: SpatialHashGrid<T>,
}

impl<T> Deref for SpatialHash3D<T> {
    type Target = SpatialHashGrid<T>;
    fn deref(&self) -> &Self::Target {
        &self.grid
    }
}

impl<T> DerefMut for SpatialHash3D<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.grid
    }
}

impl<T> SpatialHash3D<T> {
    pub fn new<V>(partitions: (usize, usize, usize), filler: V, cube_sidelength: f32) -> Self
    where V: FnMut() -> T {
        Self {
            cube_sidelength,
            bottom_left: Vector3::new(0.0, 0.0, 0.0),
            grid: SpatialHashGrid::new(partitions.0, partitions.1, partitions.2, filler),
        }
    }
    pub fn set_bottom_left(mut self, bottom_left: Vector3<f32>) -> Self {
        self.bottom_left = bottom_left;
        self
    }

    pub fn iter_cubes_mut(
        &mut self,
        min: Vector3<f32>,
        max: Vector3<f32>,
    ) -> BoxIteratorMut<'_, T> {
        let min = min - self.bottom_left;
        let max = max - self.bottom_left;
        let min = Vector3::new((min / self.cube_sidelength).x as u32, (min / self.cube_sidelength).y as u32, (min / self.cube_sidelength).z as u32);
        let max = Vector3::new((max / self.cube_sidelength).x as u32, (max / self.cube_sidelength).y as u32, (max / self.cube_sidelength).z as u32);
        self.grid.iter_cubes_mut(min, max)
    }

    pub fn iter_cubes(&self, min: Vector3<f32>, max: Vector3<f32>) -> BoxIterator<'_, T> {
        let min = min - self.bottom_left;
        let max = max - self.bottom_left;
        let min = Vector3::new((min / self.cube_sidelength).x as u32, (min / self.cube_sidelength).y as u32, (min / self.cube_sidelength).z as u32);
        let max = Vector3::new((max / self.cube_sidelength).x as u32, (max / self.cube_sidelength).y as u32, (max / self.cube_sidelength).z as u32);
        self.grid.iter_cubes(min, max)
    }

    pub fn iter_cube_indices(&self, min: Vector3<f32>, max: Vector3<f32>) -> BoxIdxIterator {
        let min = min - self.bottom_left;
        let max = max - self.bottom_left;
        let min = Vector3::new((min / self.cube_sidelength).x as u32, (min / self.cube_sidelength).y as u32, (min / self.cube_sidelength).z as u32);
        let max = Vector3::new((max / self.cube_sidelength).x as u32, (max / self.cube_sidelength).y as u32, (max / self.cube_sidelength).z as u32);
        self.grid.iter_cube_indices(min, max)
    }

    pub fn pos_to_index(&self, pos: Vector3<f32>) -> Option<usize> {
        let p = Vector3::new((pos / self.cube_sidelength).x as u32, (pos / self.cube_sidelength).y as u32, (pos / self.cube_sidelength).z as u32);
        self.grid.pos_to_index(p)
    }

    pub fn clear(&mut self, clear_fn: &mut impl FnMut(&mut T)) {
        let dims = self.grid.size();
        let dims = Vector3::new(dims.x as f32, dims.y as f32, dims.z as f32);
        let top_left = self.bottom_left + dims * self.cube_sidelength;
        let top_left = Vector3::new((top_left.x / self.cube_sidelength) as u32, (top_left.y / self.cube_sidelength) as u32, (top_left.z / self.cube_sidelength) as u32);

        let bottom_left = self.bottom_left - Vector3::new(1.0, 1.0, 1.0);
        let bottom_left = Vector3::new((bottom_left.x / self.cube_sidelength) as u32, (bottom_left.y / self.cube_sidelength) as u32, (bottom_left.z / self.cube_sidelength) as u32);

        self.grid.iter_cubes_mut(bottom_left, top_left).for_each(|(_, _, v)| clear_fn(v));
    }
}
