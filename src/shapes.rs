use ash::vk::{self, VertexInputAttributeDescription, VertexInputBindingDescription};
use glm::{Vec2, Vec3};
use std::mem::offset_of;

#[repr(C)]
pub struct UBO {
    pub model: glm::Mat4,
    pub view: glm::Mat4,
    pub proj: glm::Mat4,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertex2D {
    position: Vec2,
    color: Vec3,
    tex_coord: Vec2,
}

impl Vertex2D {
    pub fn new(position: Vec2, color: Vec3, tex_coord: Option<Vec2>) -> Self {
        Self {
            position,
            color,
            tex_coord: tex_coord.unwrap_or(position),
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
                format: vk::Format::R32G32_SFLOAT,
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
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex2D, tex_coord) as u32,
            },
        ]
    }
}

pub trait Shape {
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
}

pub struct Triangle {
    vertices: Vec<Vertex2D>,
}

impl Shape for Triangle {
    fn get_vertices(&self) -> &[Vertex2D] {
        &self.vertices
    }
}

impl Triangle {
    pub fn new(vertices: Vec<Vertex2D>) -> Self {
        Self { vertices }
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            vertices: vec![
                Vertex2D::new(Vec2::new(0.0, -0.5), Vec3::new(1.0, 0.0, 0.0), None),
                Vertex2D::new(Vec2::new(0.5, 0.5), Vec3::new(0.0, 1.0, 0.0), None),
                Vertex2D::new(Vec2::new(-0.5, 0.5), Vec3::new(0.0, 0.0, 1.0), None),
            ],
        }
    }
}

pub struct Rectangle {
    vertices: [Vertex2D; 4],
}

impl Rectangle {
    pub fn new(vertices: [Vertex2D; 4]) -> Self {
        Self { vertices }
    }
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            vertices: [
                Vertex2D::new(
                    Vec2::new(-0.25, -0.25),
                    Vec3::new(1.0, 0.0, 0.0),
                    Some(Vec2::new(0.0, 0.0)),
                ),
                Vertex2D::new(
                    Vec2::new(0.25, -0.25),
                    Vec3::new(0.0, 1.0, 0.0),
                    Some(Vec2::new(1.0, 0.0)),
                ),
                Vertex2D::new(
                    Vec2::new(0.25, 0.25),
                    Vec3::new(0.0, 0.0, 1.0),
                    Some(Vec2::new(1.0, 1.0)),
                ),
                Vertex2D::new(
                    Vec2::new(-0.25, 0.25),
                    Vec3::new(1.0, 1.0, 1.0),
                    Some(Vec2::new(0.0, 1.0)),
                ),
            ],
        }
    }
}

impl Shape for Rectangle {
    fn get_vertices(&self) -> &[Vertex2D] {
        &self.vertices
    }
    fn indices(&self) -> Vec<u32> {
        vec![0, 1, 2, 2, 3, 0]
    }
}

pub fn mobjects_to_vertices_and_indices(mobjects: &[Box<dyn Shape>]) -> (Vec<Vertex2D>, Vec<u32>) {
    let mut vertices: Vec<Vertex2D> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    for i in (0..mobjects.len()) {
        let current_length = indices.len() as u32;
        let shape_vertices = mobjects[i].vertices2d();
        let shape_indices = mobjects[i].indices();
        shape_vertices.iter().for_each(|f| vertices.push(f.clone()));
        shape_indices
            .iter()
            .for_each(|i| indices.push(*i + current_length));
    }
    (vertices, indices)
}
