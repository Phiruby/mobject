use ash::Device;
use ash::vk::{
    Buffer, DeviceMemory, PhysicalDeviceMemoryProperties
};
use nalgebra_glm as glm;
use nalgebra_glm::{Vec2, Vec3};
use std::{os::raw::c_void};
use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shapes::{Vertex2D, UBO, Shape, BuiltShape}};
use crate::define_shape;

define_shape!(
    pub struct Points {
        vertices: Vec<Vertex2D>,
        indices: Vec<u32>,
    }
);

impl Default for Points {
    fn default() -> Self {
        Self::new(vec![
            Vertex2D::new(
                Vec3::new(-0.25, -0.25, -0.90),
                Vec3::new(1.0, 1.0, 1.0),
                Some(Vec2::new(0.0, 0.0)),
            ),
            Vertex2D::new(
                Vec3::new(0.00, -0.25, 0.20),
                Vec3::new(0.0, 1.0, 0.0),
                Some(Vec2::new(0.5, 0.0)),
            ),
            Vertex2D::new(
                Vec3::new(0.25, -0.25, 0.00),
                Vec3::new(0.0, 0.0, 1.0),
                Some(Vec2::new(1.0, 0.0)),
            ),
            // Row 1 (middle)
            Vertex2D::new(
                Vec3::new(-0.25, 0.00, -0.15),
                Vec3::new(1.0, 1.0, 0.0),
                Some(Vec2::new(0.0, 0.5)),
            ),
            Vertex2D::new(
                Vec3::new(0.00, 0.00, 0.85), // center peak
                Vec3::new(1.0, 1.0, 1.0),
                Some(Vec2::new(0.5, 0.5)),
            ),
            Vertex2D::new(
                Vec3::new(0.25, 0.00, -0.65),
                Vec3::new(0.0, 1.0, 1.0),
                Some(Vec2::new(1.0, 0.5)),
            ),
            // Row 2 (top)
            Vertex2D::new(
                Vec3::new(-0.25, 0.25, 0.00),
                Vec3::new(1.0, 0.0, 1.0),
                Some(Vec2::new(0.0, 1.0)),
            ),
            Vertex2D::new(
                Vec3::new(0.00, 0.25, 0.20),
                Vec3::new(0.5, 0.5, 0.5),
                Some(Vec2::new(0.5, 1.0)),
            ),
            Vertex2D::new(
                Vec3::new(0.25, 0.25, 0.90),
                Vec3::new(0.2, 0.8, 1.0),
                Some(Vec2::new(1.0, 1.0)),
            ),
        ])
    }
}
