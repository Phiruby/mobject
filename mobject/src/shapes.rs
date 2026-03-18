pub mod surface;
pub mod primitives;
pub mod generator;

pub use primitives::*;
pub use generator::*;
pub use surface::Points;
use ash::Device;
use ash::vk::{
    self, Buffer, DeviceMemory, PhysicalDeviceMemoryProperties, VertexInputAttributeDescription,
    VertexInputBindingDescription,
};
use nalgebra_glm as glm;
use nalgebra_glm::{Vec2, Vec3};
use std::{mem::offset_of, os::raw::c_void};
use crate::pipelines::Pipelines;
use crate::{MAX_FRAMES_IN_FLIGHT};

#[repr(C)]
#[derive(Debug)]
pub struct UBO {
    pub model: glm::Mat4,
}

#[repr(C)]
#[derive(Debug)]
pub struct GlobalUBO {
    pub camera_position: glm::Vec3,
    pub _pad: u32, // 4 bytes of padding, because std140 rule states vec3 takes 16 bytes...
    pub view: glm::Mat4,
    pub proj: glm::Mat4,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertex2D {
    pub position: Vec3,
    pub color: Vec3,
    pub tex_coord: Vec2,
}

impl Vertex2D {
    pub fn new(position: Vec3, color: Vec3, tex_coord: Option<Vec2>) -> Self {
        Self {
            position,
            color,
            tex_coord: tex_coord.unwrap_or(Vec2::new(0.0, 0.0)),
        }
    }
    pub fn binding_description() -> VertexInputBindingDescription {
        VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Vertex2D>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn attribute_descriptions() -> [VertexInputAttributeDescription; 3] {
        [
            VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex2D, position) as u32,
            },
            VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex2D, color) as u32,
            },
            VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex2D, tex_coord) as u32,
            },
        ]
    }
}

pub trait BuiltShape {
    fn get_pipeline(&self) -> Pipelines;
    fn vertices2d(&self) -> &[Vertex2D] {
        self.get_vertices()
    }
    fn get_vertices(&self) -> &[Vertex2D];
    fn indices(&self) -> &[u32];
    fn vertices_and_indices(&self) -> (&[Vertex2D], &[u32]) {
        (self.get_vertices(), self.indices())
    }
    fn get_uniform_buffer(
        &self,
    ) -> (
        &[Buffer; MAX_FRAMES_IN_FLIGHT as usize],
        &[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize],
        &[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize],
    );
    fn get_ubo_contents(&self) -> &UBO;
}

pub trait Shape {
    fn build(
        self: Box<Self>,
        device: &Device,
        mem_properties: PhysicalDeviceMemoryProperties,
    ) -> Box<dyn BuiltShape>;
}


pub fn mobjects_to_vertices_and_indices(
    mobjects: &[&Box<dyn BuiltShape>],
) -> (Vec<Vertex2D>, Vec<u32>) {
    let mut vertices: Vec<Vertex2D> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    for i in (0..mobjects.len()) {
        let current_length = vertices.len() as u32;
        let shape_vertices = mobjects[i].vertices2d();
        let shape_indices = mobjects[i].indices();
        shape_vertices.iter().for_each(|f| vertices.push(f.clone()));
        shape_indices
            .iter()
            .for_each(|&i| indices.push(i + current_length));
    }
    (vertices, indices)
}
