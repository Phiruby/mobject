use ash::Device;
use ash::vk::{
    Buffer, DeviceMemory, PhysicalDeviceMemoryProperties
};
use nalgebra_glm as glm;
use nalgebra_glm::Vec3;
use obj::{Obj, TexturedVertex, load_obj};
use std::{os::raw::c_void};

use crate::shapes::ShapeIntent;
use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shapes::{RenderVertex, UBO, Shape, BuiltShape}};
use crate::{define_shape, shapes};
use crate::pipelines::Pipelines;
use crate::shapes::Manifold;
define_shape!(
    pub struct Triangle {
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
    },
    Pipelines::Primitive,
    Manifold::TwoD
);

define_shape!(
    pub struct Cube {
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
    },
    Pipelines::Primitive,
    Manifold::ThreeD
);

define_shape!(
    pub struct ObjModel {
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
    },
    Pipelines::Primitive,
    Manifold::ThreeD
);


define_shape!(
    pub struct Rectangle {
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
    },
    Pipelines::Primitive,
    Manifold::TwoD
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
                RenderVertex::new(pos, col, norm, None)
            })
            .collect();

        Self::new().with_vertices(vertices).with_indices(vec![0, 1, 2])
    }
}


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

        let vertices: Vec<RenderVertex> = model.vertices
            .iter()
            .zip(positions.into_iter())
            .map(|(vert, pos)| {
                    RenderVertex::new(
                        pos,
                        glm::make_vec3(&[0.0, 0.0, 0.0]),
                        glm::make_vec3(&vert.normal),
                        Some(glm::make_vec2(&[vert.texture[0], 1.0 - vert.texture[1]]))
                    )
                }
            )
            .collect();

        let indices = model.indices;
        Self::new().with_indices(indices).with_vertices(vertices)
    }
}


impl Rectangle {
    pub fn load(vertices: [RenderVertex; 4]) -> Self {
        Self::new().with_indices(vec![0, 1, 2, 2, 3, 0]).with_vertices(vertices.to_vec())
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

        let vertices: Vec<RenderVertex> = positions
            .into_iter()
            .zip(normals.into_iter())
            .map(|(pos, norm)|
                RenderVertex::new(
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

impl Default for Cube {
    fn default() -> Self {
        let positions = vec![
            Vec3::new(-0.25, -0.25,  2.0),
            Vec3::new( 0.25, -0.25,  2.0),
            Vec3::new( 0.25,  0.25,  2.0),
            Vec3::new(-0.25,  0.25,  2.0),

            Vec3::new(-0.25, -0.25, 1.5),
            Vec3::new( 0.25, -0.25, 1.5),
            Vec3::new( 0.25,  0.25, 1.5),
            Vec3::new(-0.25,  0.25, 1.5),
        ];

        let indices: Vec<usize> = vec![
            // Front
            0, 1, 2, 2, 3, 0,
            // Back
            5, 4, 7, 7, 6, 5,
            // Left
            4, 0, 3, 3, 7, 4,
            // Right
            1, 5, 6, 6, 2, 1,
            // Top
            3, 2, 6, 6, 7, 3,
            // Bottom
            4, 5, 1, 1, 0, 4,
        ];

        let normals = shapes::compute_normals(&indices, &positions);

        let vertices: Vec<RenderVertex> = positions
            .into_iter()
            .zip(normals.into_iter())
            .map(|(pos, norm)| {
                RenderVertex::new(
                    pos,
                    Vec3::new(1.0, 0.0, 0.0), // color
                    norm,
                    None,
                )
            })
            .collect();
        let u32_indices = indices.iter().map(|&x| x as u32).collect();
        Self::new()
            .with_vertices(vertices)
            .with_indices(u32_indices)
    }
}

impl Cube {
    pub fn example(subdivisions: usize) -> Self {
        let half = 0.25; // smaller cube (total size = 0.5)
        let center = Vec3::new(0.0, 0.0, -0.1);

        let mut positions = Vec::new();
        let mut indices = Vec::new();

        let mut add_face = |origin: Vec3, u_dir: Vec3, v_dir: Vec3| {
            let base_index = positions.len();

            for i in 0..=subdivisions {
                for j in 0..=subdivisions {
                    let u = i as f32 / subdivisions as f32;
                    let v = j as f32 / subdivisions as f32;

                    positions.push(origin + u_dir * u + v_dir * v);
                }
            }

            for i in 0..subdivisions {
                for j in 0..subdivisions {
                    let i0 = base_index + i * (subdivisions + 1) + j;
                    let i1 = i0 + 1;
                    let i2 = i0 + (subdivisions + 1);
                    let i3 = i2 + 1;

                    indices.extend_from_slice(&[
                        i0, i3, i1,
                        i3, i0, i2,
                    ]);
                }
            }
        };

        // Cube min/max (same extent in all axes)
        let min = center - Vec3::new(half, half, half);
        let max = center + Vec3::new(half, half, half);

        let dx = Vec3::new(max.x - min.x, 0.0, 0.0);
        let dy = Vec3::new(0.0, max.y - min.y, 0.0);
        let dz = Vec3::new(0.0, 0.0, max.z - min.z);

        // 6 faces
        add_face(Vec3::new(min.x, min.y, min.z), dx, dy); // bottom
        add_face(Vec3::new(min.x, min.y, max.z), dx, dy); // top
        add_face(Vec3::new(min.x, min.y, min.z), dx, dz); // front
        add_face(Vec3::new(min.x, max.y, min.z), dx, dz); // back
        add_face(Vec3::new(min.x, min.y, min.z), dy, dz); // left
        add_face(Vec3::new(max.x, min.y, min.z), dy, dz); // right

        let normals = shapes::compute_normals(&indices, &positions);

        let vertices: Vec<RenderVertex> = positions
            .into_iter()
            .zip(normals.into_iter())
            .map(|(pos, norm)| {
                RenderVertex::new(
                    pos,
                    Vec3::new(1.0, 0.0, 0.0),
                    norm,
                    None,
                )
            })
            .collect();
        let u32_indices = indices.iter().map(|&x| x as u32).collect();

        Self::new()
            .with_vertices(vertices)
            .with_indices(u32_indices)
    }

    pub fn edge_indices_top_front(subdivisions: usize) -> Vec<usize> {
        let verts_per_face = (subdivisions + 1) * (subdivisions + 1);

        let top_face_offset = verts_per_face; // second face added

        let mut indices = Vec::new();

        for i in 0..=subdivisions {
            // front row of top face (j = 0)
            let idx = top_face_offset + i * (subdivisions + 1);
            indices.push(idx);
        }

        indices
    }
}
pub fn create_baseplate() -> ShapeIntent {
    let width = 5.0;
    let depth = 0.1;
    let height = 5.0;

    let hw = width / 2.0;
    let hd = depth / 2.0;
    let h  = height / 2.0;

    let positions = vec![
        Vec3::new(-hw,  h, -hd),
        Vec3::new( hw,  h, -hd),
        Vec3::new( hw,  h,  hd),
        Vec3::new(-hw,  h,  hd),

        Vec3::new(-hw, -h, -hd),
        Vec3::new( hw, -h, -hd),
        Vec3::new( hw, -h,  hd),
        Vec3::new(-hw, -h,  hd),
    ];

    let indices: Vec<u32> = vec![
        0, 1, 2, 2, 3, 0,
        5, 4, 7, 7, 6, 5,
        4, 0, 3, 3, 7, 4,
        1, 5, 6, 6, 2, 1,
        4, 5, 1, 1, 0, 4,
        // top
        3, 6, 2, 6, 3, 7,
    ];

    let normals = shapes::compute_normals(&indices.iter().map(|&i| i as usize).collect::<Vec<_>>(), &positions);

    let vertices: Vec<RenderVertex> = positions
        .into_iter()
        .zip(normals.into_iter())
        .map(|(pos, norm)| {
            RenderVertex::new(
                pos,
                Vec3::new(0.6, 0.6, 0.6), // neutral baseplate color
                norm,
                None,
            )
        })
        .collect();

    Cube::new()
        .with_vertices(vertices)
        .with_indices(indices)
        .finish_construction()
        .make_static()
}
