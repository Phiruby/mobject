extern crate proc_macro;
use ash::Device;
use ash::vk::{
    self, Buffer, DeviceMemory, PhysicalDeviceMemoryProperties, VertexInputAttributeDescription,
    VertexInputBindingDescription,
};
use glm::{Vec2, Vec3};
use std::{char::MAX, mem::offset_of, os::raw::c_void};

use crate::{MAX_FRAMES_IN_FLIGHT, buffers};

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
    position: Vec3,
    color: Vec3,
    tex_coord: Vec2,
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
    fn vertices2d(&self) -> &[Vertex2D] {
        self.get_vertices()
    }
    fn get_vertices(&self) -> &[Vertex2D];
    fn indices(&self) -> Vec<u32> {
        (0..self.get_vertices().len() as u32).collect()
    }
    fn vertices_and_indices(&self) -> (&[Vertex2D], Vec<u32>) {
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

pub struct Triangle {
    vertices: Vec<Vertex2D>,
    uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
    ubo: UBO,
}

impl Triangle {
    pub fn new(vertices: Vec<Vertex2D>) -> Self {
        Self {
            vertices,
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

impl Shape for Triangle {
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
        let triangle = Self {
            vertices: self.vertices,
            uniform_buffers: Some(uniform_buffers),
            uniform_buffer_memories: Some(uniform_buffer_memories),
            uniform_mapped_memories: Some(uniform_mapped_memories),
            ubo: self.ubo,
        };
        Box::new(triangle)
    }
}

impl BuiltShape for Triangle {
    fn get_vertices(&self) -> &[Vertex2D] {
        &self.vertices
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
impl Default for Triangle {
    fn default() -> Self {
        Self::new(vec![
            Vertex2D::new(Vec3::new(0.0, -0.5, -0.5), Vec3::new(1.0, 0.0, 0.0), None),
            Vertex2D::new(Vec3::new(0.5, 0.5, -0.5), Vec3::new(0.0, 1.0, 0.0), None),
            Vertex2D::new(Vec3::new(-0.5, 0.5, -0.5), Vec3::new(0.0, 0.0, 1.0), None),
        ])
    }
}

pub struct Sphere {
    vertices: Vec<Vertex2D>,
    uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
    ubo: UBO,
}

impl Sphere {
    pub fn new(origin: glm::Vec3, radius: f32) -> Self {
        todo!()
    }
}

pub struct Rectangle {
    vertices: [Vertex2D; 4],
    uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
    ubo: UBO,
}

impl Rectangle {
    pub fn new(vertices: [Vertex2D; 4]) -> Self {
        Self {
            vertices,
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

impl Shape for Rectangle {
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
        let triangle = Self {
            vertices: self.vertices,
            uniform_buffers: Some(uniform_buffers),
            uniform_buffer_memories: Some(uniform_buffer_memories),
            uniform_mapped_memories: Some(uniform_mapped_memories),
            ubo: self.ubo,
        };
        Box::new(triangle)
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self::new([
            Vertex2D::new(
                Vec3::new(-0.25, -0.25, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Some(Vec2::new(0.0, 0.0)),
            ),
            Vertex2D::new(
                Vec3::new(0.25, -0.25, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Some(Vec2::new(1.0, 0.0)),
            ),
            Vertex2D::new(
                Vec3::new(0.25, 0.25, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                Some(Vec2::new(1.0, 1.0)),
            ),
            Vertex2D::new(
                Vec3::new(-0.25, 0.25, 0.0),
                Vec3::new(1.0, 1.0, 1.0),
                Some(Vec2::new(0.0, 1.0)),
            ),
        ])
    }
}

impl BuiltShape for Rectangle {
    fn get_vertices(&self) -> &[Vertex2D] {
        &self.vertices
    }
    fn indices(&self) -> Vec<u32> {
        vec![0, 1, 2, 2, 3, 0]
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

pub fn mobjects_to_vertices_and_indices(
    mobjects: &[Box<dyn BuiltShape>],
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
            .for_each(|i| indices.push(*i + current_length));
    }
    (vertices, indices)
}
