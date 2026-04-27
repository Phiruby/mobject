use ash::Device;
use ash::vk::{
    Buffer, DeviceMemory, PhysicalDeviceMemoryProperties
};
use nalgebra_glm as glm;
use nalgebra_glm::Vec3;
use obj::{Obj, TexturedVertex, load_obj};
use std::{os::raw::c_void};

use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shapes::{RenderVertex, UBO, Shape, BuiltShape}};
use crate::{define_shape, shapes};
use crate::pipelines::Pipelines;
use crate::physics::constraints::*;

define_shape!(
    pub struct Cloth {
        vertices: Vec<RenderVertex>,
        indices: Vec<u32>,
    },
    Pipelines::Primitive
);

fn generate_grid(width: usize, height: usize, spacing: f32) -> (Vec<Vec3>, Vec<usize>) {
    let mut positions = vec![];
    let mut indices = vec![];

    // positions
    for y in 0..height {
        for x in 0..width {
            positions.push(Vec3::new(
                x as f32 * spacing,
                // 0.0,
                y as f32 * spacing,
                0.0
            ));
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

    (positions, indices)
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

fn create_bend_constraints(
    width: usize,
    height: usize,
    positions: &[Vec3],
    k: f32,
) -> Vec<PhysicsConstraint> {
    let mut constraints = vec![];

    let idx = |x: usize, y: usize| y * width + x;

    for y in 0..height - 1 {
        for x in 0..width - 1 {
            let i0 = idx(x, y);
            let i1 = idx(x + 1, y);
            let i2 = idx(x, y + 1);
            let i3 = idx(x + 1, y + 1);
            if i0 >= 2 {continue;}

            let x1 = positions[i1];
            let x2 = positions[i2];
            let x3 = positions[i0];
            let x4 = positions[i3];

            let e = x2 - x1;
            let n1 = nalgebra_glm::cross(&e, &(x3 - x1)).normalize();
            let n2 = nalgebra_glm::cross(&e, &(x4 - x1)).normalize();
            let phi = nalgebra_glm::dot(&n1, &n2).clamp(-1.0, 1.0).acos();
            // dbg!(x1, x2, x3, x4, n1, n2, phi);

            constraints.push(PhysicsConstraint::Bend(BendConstraint {
                vert_ind1: i1,
                vert_ind2: i2,
                vert_ind3: i0,
                vert_ind4: i3,
                k,
                phi,
            }));
        }
    }

    constraints
}

impl Default for Cloth {
    fn default() -> Self {
        let width = 10;
        let height = 10;
        let spacing = 0.1;

        let (positions, indices) = generate_grid(width, height, spacing);

        let normals = shapes::compute_normals(&indices, &positions);

        let vertices: Vec<RenderVertex> = positions
            .iter()
            .zip(normals.iter())
            .map(|(pos, norm)| {
                RenderVertex::new(
                    *pos,
                    Vec3::new(0.7, 0.7, 1.0),
                    *norm,
                    None,
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

        // constraints.extend(create_bend_constraints(
        //     width,
        //     height,
        //     &positions,
        //     1.0,
        // ));

        Self::new()
        .with_vertices(vertices)
        .with_indices(indices.iter().map(|&i| i as u32).collect(),)
        .with_constraints(constraints)
    }
}
