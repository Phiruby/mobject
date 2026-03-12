use ash::Device;
use ash::vk::{
    Buffer, DeviceMemory, PhysicalDeviceMemoryProperties
};
use nalgebra_glm as glm;
use nalgebra_glm::{Vec2, Vec3};
use obj::{Obj, TexturedVertex, load_obj};
use std::{os::raw::c_void};

use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shapes::{Vertex2D, UBO, Shape, BuiltShape}};
use crate::define_shape;
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
        Self::new(vec![
            Vertex2D::new(Vec3::new(0.0, -0.5, 0.25), Vec3::new(1.0, 0.0, 0.0), None),
            Vertex2D::new(Vec3::new(0.5, 0.5, 0.25), Vec3::new(0.0, 1.0, 0.0), None),
            Vertex2D::new(Vec3::new(-0.5, 0.5, 0.25), Vec3::new(0.0, 0.0, 1.0), None),
        ])
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
        Self ::with_indices(vertices, indices)
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
        Self::load([
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
