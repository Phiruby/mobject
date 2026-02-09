use ash::Device;
use ash::vk::{
    Buffer, DeviceMemory, PhysicalDeviceMemoryProperties
};
use nalgebra_glm as glm;
use nalgebra_glm::{Vec2, Vec3};
use obj::{Obj, TexturedVertex, load_obj};
use std::{os::raw::c_void};

use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shapes::{Vertex2D, UBO, Shape, BuiltShape}};

pub struct Triangle {
    vertices: Vec<Vertex2D>,
    indices: Vec<u32>,
    uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
    ubo: UBO,
}

impl Triangle {
    pub fn new(vertices: Vec<Vertex2D>) -> Self {
        Self {
            vertices,
            indices: (0..3).collect(),
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
            indices: self.indices,
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
impl Default for Triangle {
    fn default() -> Self {
        Self::new(vec![
            Vertex2D::new(Vec3::new(0.0, -0.5, 0.25), Vec3::new(1.0, 0.0, 0.0), None),
            Vertex2D::new(Vec3::new(0.5, 0.5, 0.25), Vec3::new(0.0, 1.0, 0.0), None),
            Vertex2D::new(Vec3::new(-0.5, 0.5, 0.25), Vec3::new(0.0, 0.0, 1.0), None),
        ])
    }
}

pub struct ObjModel {
    vertices: Vec<Vertex2D>,
    indices: Vec<u32>,
    uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
    ubo: UBO,
}

impl ObjModel {
    pub fn new(obj_file: &str) -> Self {
        let input = std::io::BufReader::new(std::fs::File::open(obj_file).unwrap());
        let model: Obj<TexturedVertex, u32> = load_obj(input).unwrap();
        dbg!(model.vertices.len());
        let vertices: Vec<Vertex2D> = model
            .vertices
            .iter()
            .map(|vert| {
                let pos = glm::make_vec3(&vert.position);
                // in vulkan, 0 implies top of image; for obj files, 0 implies bottom. so 1.0 - ...
                let tex_coord = glm::make_vec2(&[vert.texture[0], 1.0 - vert.texture[1]]);
                let color = glm::vec3(0.0, 0.0, 0.0);
                Vertex2D::new(pos, color, Some(tex_coord))
            })
            .collect();
        let indices = model.indices;
        dbg!(vertices.len(), indices.len());
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

impl Shape for ObjModel {
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
        let obj = Self {
            vertices: self.vertices,
            indices: self.indices,
            uniform_buffers: Some(uniform_buffers),
            uniform_buffer_memories: Some(uniform_buffer_memories),
            uniform_mapped_memories: Some(uniform_mapped_memories),
            ubo: self.ubo,
        };
        Box::new(obj)
    }
}

impl BuiltShape for ObjModel {
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

pub struct Rectangle {
    vertices: [Vertex2D; 4],
    indices: [u32; 6],
    uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
    uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
    ubo: UBO,
}

impl Rectangle {
    pub fn new(vertices: [Vertex2D; 4]) -> Self {
        Self {
            vertices,
            indices: [0, 1, 2, 2, 3, 0],
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
            indices: self.indices,
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
