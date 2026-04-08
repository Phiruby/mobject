use ash::Device;
use ash::vk::{
    Buffer, DeviceMemory, PhysicalDeviceMemoryProperties
};
use nalgebra_glm as glm;
use nalgebra_glm::{Vec2, Vec3};
use std::{os::raw::c_void};
use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shapes::{Vertex2D, UBO, Shape, BuiltShape}, pipelines::Pipelines};
use crate::{define_shape, shapes};

define_shape!(
    pub struct Points {
        vertices: Vec<Vertex2D>,
        indices: Vec<u32>,
    },
    Pipelines::Bezier
);

impl Default for Points {
    fn default() -> Self {
        let positions: Vec<Vec3> = vec![
            Vec3::new(-0.25, -0.25, -0.90),
            Vec3::new(0.00, -0.25, 0.20),
            Vec3::new(0.25, -0.25, 0.00),

            Vec3::new(-0.25, 0.00, -0.15),
            Vec3::new(0.00, 0.00, 0.85),
            Vec3::new(0.25, 0.00, -0.65),

            Vec3::new(-0.25, 0.25, 0.00),
            Vec3::new(0.00, 0.25, 0.20),
            Vec3::new(0.25, 0.25, 0.90),
        ];
        let indices: Vec<usize> = vec![
            0, 1, 3,  1, 4, 3,
            1, 2, 4,  2, 5, 4,
            3, 4, 6,  4, 7, 6,
            4, 5, 7,  5, 8, 7,
        ];
        let normals = shapes::compute_normals(&indices, &positions);
        let vertices: Vec<Vertex2D> = positions
            .into_iter()
            .zip(normals.into_iter())
            .enumerate()
            .map(|(i, (pos, normal))| {
                let uv = Vec2::new(
                    (i % 3) as f32 / 2.0,
                    (i / 3) as f32 / 2.0,
                );

                Vertex2D::new(
                    pos,
                    glm::make_vec3(&[0.0, 1.0, 0.0]),
                    normal,
                    Some(uv)
                )
            })
            .collect();
        Self::new(vertices)
    }
}
