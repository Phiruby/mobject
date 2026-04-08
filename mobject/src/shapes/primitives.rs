use ash::Device;
use ash::vk::{
    Buffer, DeviceMemory, PhysicalDeviceMemoryProperties
};
use nalgebra_glm as glm;
use nalgebra_glm::{Vec2, Vec3};
use obj::{Obj, TexturedVertex, load_obj};
use std::{os::raw::c_void};

use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shapes::{Vertex2D, UBO, Shape, BuiltShape}};
use crate::{define_shape, shapes};
use crate::pipelines::Pipelines;
define_shape!(
    pub struct Triangle {
        vertices: Vec<Vertex2D>,
        indices: Vec<u32>,
    },
    Pipelines::Primitive
);
impl Default for Triangle {
    fn default() -> Self {
        let positions = vec![
            Vec3::new(0.0, -0.5, 0.25),
            Vec3::new(0.5, 0.5, 0.25),
            Vec3::new(-0.5, 0.5, 0.25)
        ];
        let normals = shapes::compute_normals(&[0, 1, 2], &positions);
        let colors = vec![
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0)
        ];
        let vertices = positions
            .into_iter()
            .zip(normals.into_iter())
            .zip(colors.into_iter())
            .map(|((pos, norm), col)| {
                Vertex2D::new(pos, col, norm, None)
            })
            .collect();

        Self::new(vertices)
    }
}

define_shape!(
    pub struct ObjModel {
        vertices: Vec<Vertex2D>,
        indices: Vec<u32>,
    },
    Pipelines::Primitive
);

impl ObjModel {
    pub fn load(obj_file: &str) -> Self {
        let input = std::io::BufReader::new(std::fs::File::open(obj_file).unwrap());
        let model: Obj<TexturedVertex, u32> = load_obj(input).unwrap();

        let positions: Vec<Vec3> = model
            .vertices
            .iter()
            .map(|vert| {
                glm::make_vec3(&vert.position)
            })
            .collect();

        let vertices: Vec<Vertex2D> = model.vertices
            .iter()
            .zip(positions.into_iter())
            .map(|(vert, pos)| {
                    Vertex2D::new(
                        pos,
                        glm::make_vec3(&[0.0, 0.0, 0.0]),
                        glm::make_vec3(&vert.normal),
                        Some(glm::make_vec2(&[vert.texture[0], 1.0 - vert.texture[1]]))
                    )
                }
            )
            .collect();

        let indices = model.indices;
        Self::with_indices(vertices, indices)
    }
}

define_shape!(
    pub struct Rectangle {
        vertices: [Vertex2D; 4],
        indices: Vec<u32>,
    },
    Pipelines::Primitive
);
impl Rectangle {
    pub fn load(vertices: [Vertex2D; 4]) -> Self {
        Self::with_indices(vertices, vec![0, 1, 2, 2, 3, 0])
    }
}

impl Default for Rectangle {
    fn default() -> Self {

        let positions = vec![
            Vec3::new(-0.25, -0.25, 0.0),
            Vec3::new(0.25, -0.25, 0.0),
            Vec3::new(0.25, 0.25, 0.0),
            Vec3::new(-0.25, 0.25, 0.0)
        ];
        let normals = shapes::compute_normals(&[0, 1, 2, 2, 3, 0], &positions);

        let vertices: Vec<Vertex2D> = positions
            .into_iter()
            .zip(normals.into_iter())
            .map(|(pos, norm)|
                Vertex2D::new(
                    pos,
                    Vec3::new(1.0, 0.0, 0.0),
                    -norm,
                    None
                )
            )
            .collect();

        Self::load(vertices.try_into().unwrap())
    }
}
