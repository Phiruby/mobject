use ash::vk::{self, VertexInputAttributeDescription, VertexInputBindingDescription};
use glm::{Vec2, Vec3};
use std::mem::offset_of;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertex2D {
    position: Vec2,
    color: Vec3,
}

impl Vertex2D {
    pub fn binding_description() -> VertexInputBindingDescription {
        VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Vertex2D>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn attribute_descriptions() -> [VertexInputAttributeDescription; 2] {
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
        ]
    }
}

pub trait Shape {
    fn vertices2d(&self) -> &Vec<Vertex2D>;
}

pub struct Triangle {
    vertices: Vec<Vertex2D>,
}

impl Shape for Triangle {
    fn vertices2d(&self) -> &Vec<Vertex2D> {
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
                Vertex2D {
                    position: Vec2::new(0.0, -0.5),
                    color: Vec3::new(1.0, 0.0, 0.0),
                },
                Vertex2D {
                    position: Vec2::new(0.5, 0.5),
                    color: Vec3::new(0.0, 1.0, 0.0),
                },
                Vertex2D {
                    position: Vec2::new(-0.5, 0.5),
                    color: Vec3::new(0.0, 0.0, 1.0),
                },
            ],
        }
    }
}
