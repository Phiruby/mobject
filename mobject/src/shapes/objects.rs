use ash::Device;
use ash::vk::{
    Buffer, DeviceMemory, PhysicalDeviceMemoryProperties
};
use nalgebra_glm::{self as glm, Vec2};
use nalgebra_glm::Vec3;
use std::{os::raw::c_void};

use crate::shapes::ShapeIntent;
use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shapes::{RenderVertex, UBO, Shape, BuiltShape}};
use crate::{define_shape, shapes};
use crate::pipelines::Pipelines;
use crate::physics::constraints::*;
use crate::shapes::Manifold;

define_shape!(
    pub struct Cloth {
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
    },
    Pipelines::Primitive,
    Manifold::TwoD
);

fn generate_grid(width: usize, height: usize, spacing: f32) -> (Vec<Vec3>, Vec<usize>, Vec<Vec2>) {
    let mut positions = vec![];
    let mut tex_coords: Vec<Vec2> = vec![];
    let mut indices = vec![];
    let half_w = (width.saturating_sub(1)) as f32 * spacing * 0.5;
    let half_h = (height.saturating_sub(1)) as f32 * spacing * 0.5;
    // positions
    for y in 0..height {
        for x in 0..width {
            positions.push(Vec3::new(
                x as f32 * spacing - half_w,
                y as f32 * spacing - half_h,
                1.0
            ));
            tex_coords.push(Vec2::new(x as f32 / width as f32, y as f32 / height as f32));
        }
    }

    // triangles
    for y in 0..height - 1 {
        for x in 0..width - 1 {
            let i = y * width + x;

            let i0 = i;
            let i1 = i + 1;
            let i2 = i + width;
            let i3 = i + width + 1;

            // triangle 1
            indices.extend_from_slice(&[i0, i2, i1]);

            // triangle 2
            indices.extend_from_slice(&[i1, i2, i3]);
        }
    }

    (positions, indices, tex_coords)
}

fn create_stretch_constraints(
    width: usize,
    height: usize,
    positions: &[Vec3],
    k: f32,
) -> Vec<PhysicsConstraint> {
    let mut constraints = vec![];

    let idx = |x: usize, y: usize| y * width + x;

    for y in 0..height {
        for x in 0..width {
            if x < width - 1 {
                let i1 = idx(x, y);
                let i2 = idx(x + 1, y);

                let l0 = (positions[i1] - positions[i2]).magnitude();

                constraints.push(PhysicsConstraint::Stretch(StretchConstraint {
                    vert_ind1: i1,
                    vert_ind2: i2,
                    l0,
                    k,
                }));
            }

            if y < height - 1 {
                let i1 = idx(x, y);
                let i2 = idx(x, y + 1);

                let l0 = (positions[i1] - positions[i2]).magnitude();

                constraints.push(PhysicsConstraint::Stretch(StretchConstraint {
                    vert_ind1: i1,
                    vert_ind2: i2,
                    l0,
                    k,
                }));
            }
        }
    }

    constraints
}



impl Cloth {
    pub fn example() -> ShapeIntent {
        let width = 10;
        let height = 10;
        let spacing = 0.1;

        let (positions, indices, tex_coords) = generate_grid(width, height, spacing);

        let normals = shapes::compute_normals(&indices, &positions);

        let vertices: Vec<RenderVertex> = positions
            .iter()
            .zip(normals.iter())
            .zip(tex_coords.iter())
            .map(|((pos, norm), tex)| {
                RenderVertex::new(
                    *pos,
                    Vec3::new(1.0, 1.0, 1.0),
                    *norm,
                    Some(*tex),
                )
            })
            .collect();

        let mut constraints = vec![];

        constraints.extend(create_stretch_constraints(
            width,
            height,
            &positions,
            0.5,
        ));
        let total = vertices.len();
        Self::new()
        .with_vertices(vertices)
        .with_indices(indices.iter().map(|&i| i as u32).collect(),)
        .with_texture(String::from("textures/cloth.jpg"))
        .finish_construction()
        .with_constraints(constraints)
    }
}
