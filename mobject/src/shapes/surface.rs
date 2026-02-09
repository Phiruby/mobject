use ash::Device;
use ash::vk::{
    Buffer, DeviceMemory, PhysicalDeviceMemoryProperties
};
use nalgebra_glm as glm;
use nalgebra_glm::{Vec2, Vec3};
use std::{os::raw::c_void};

use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shapes::{Vertex2D, UBO, Shape, BuiltShape}};


pub struct Points {
    vertices: Vec<Vertex2D>,
    indices: Vec<u32>,
    uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
    ubo: UBO,
}
impl Points {
    fn new(vertices: Vec<Vertex2D>) -> Self {
        let indices = (0..vertices.len() as u32).collect();
        Self {
            vertices,
            indices,
            uniform_buffers: None,
            uniform_buffer_memories: None,
            uniform_mapped_memories: None,
            ubo: UBO {
                model: glm::mat4(
                    1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
                ),
            },
        }
    }
}
impl Shape for Points {
    fn build(
        self: Box<Self>,
        device: &Device,
        mem_properties: PhysicalDeviceMemoryProperties,
    ) -> Box<dyn BuiltShape> {
        let (uniform_buffers, uniform_buffer_memories, uniform_mapped_memories) =
            buffers::create_uniform_buffers::<{ MAX_FRAMES_IN_FLIGHT as usize }>(
                device,
                mem_properties,
                size_of::<UBO>() as u64,
            );
        let points = Self {
            vertices: self.vertices,
            indices: self.indices,
            uniform_buffers: Some(uniform_buffers),
            uniform_buffer_memories: Some(uniform_buffer_memories),
            uniform_mapped_memories: Some(uniform_mapped_memories),
            ubo: self.ubo,
        };
        Box::new(points)
    }
}

impl BuiltShape for Points {
    fn get_vertices(&self) -> &[Vertex2D] {
        &self.vertices
    }
    fn indices(&self) -> &[u32] {
        &self.indices
    }
    fn get_uniform_buffer(
        &self,
    ) -> (
        &[Buffer; MAX_FRAMES_IN_FLIGHT as usize],
        &[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize],
        &[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize],
    ) {
        (
            self.uniform_buffers.as_ref().unwrap(),
            self.uniform_buffer_memories.as_ref().unwrap(),
            self.uniform_mapped_memories.as_ref().unwrap(),
        )
    }
    fn get_ubo_contents(&self) -> &UBO {
        &self.ubo
    }
}
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
