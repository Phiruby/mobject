use std::collections::HashMap;

use cgmath::Vector3;
use nalgebra_glm::{Vec3, Mat3};
use shapeject::SpatialHash3D;
use spatial_hash_3d::SpatialHashGrid;
use crate::physics::constraints::{CollisionConstraint, Constraint, Equality, PhysicsConstraint};
use crate::{scene::Mobject};
use crate::physics;
use nalgebra::{Complex, Matrix3, Matrix3x2, SVD};
pub struct PBDSolver ();

/// Implementation of https://matthias-research.github.io/pages/publications/MeshlessDeformations_SIG05.pdf
fn shape_matching(
    world_positions: &mut [Vec3],
    current_com: Vec3,
    body_positions: &[Vec3],
    inv_masses: &[f32],
) {
    let m1 = world_positions
        .iter()
        .zip(body_positions)
        .zip(inv_masses.iter())
        .filter(|((_, _), m)| **m > 0.0)
        .map(
            |((w, b), &m)| nalgebra_glm::outer_product(&(w - current_com), b)
        )
        .sum::<nalgebra_glm::Mat3>();
    // TODO: can precompute this at the beginning
    let m2 = body_positions
        .iter()
        .zip(inv_masses.iter())
        .filter(|(_, m)| **m > 0.0)
        .map(|(b, &m)| nalgebra_glm::outer_product(b, b))
        .sum::<nalgebra_glm::Mat3>();

    let m2_inv = m2
        .try_inverse()
        .expect("Body space coords must be rank 3");

    let A = m1 * m2_inv;
    let svd = A.svd(true, true);
    let mut U = svd.u.unwrap();
    let V = svd.v_t.unwrap();
    let mut R = U * V;
    if R.determinant() < 0.0 {
        U.column_mut(2).scale_mut(-1.0);
        R = U * V;
    }
    for ((w, b), inv_m) in world_positions.iter_mut().zip(body_positions.iter()).zip(inv_masses.iter()) {
        // inf mass, skip
        if (*inv_m) == 0.0 {
            continue;
        }
        let p = R * b;
        *w = p + current_com;
    }

}

fn project_constraints(
    world_positions: &mut [Vec3],
    body_positions: &[Vec3],
    inv_masses: &[f32],
    constraints: &[PhysicsConstraint],
) {
    for c in constraints {
        let current_com = physics::raw_center_of_mass(world_positions, inv_masses);
        // TODO: hacking over rigid constraint
        if let PhysicsConstraint::RigidBody(_) = c {
            shape_matching(world_positions, current_com, body_positions, inv_masses);
            continue;
        }
        let ev = c.evaluate(world_positions);
        if c.equality_type() == Equality::Inequality {
            if ev > 0.0 {
                continue;
            }
        }
        if ev.abs() <= f32::EPSILON {
            continue;
        }
        let grads = c.gradient(world_positions);
        let denom = grads.iter().fold(0.0, |acc, grad| acc + inv_masses[grad.index] * grad.gradient.norm());
        let s = ev / denom;
        if s.is_nan() {
            dbg!("Ooof");
        }
        for g in grads {
            world_positions[g.index] -= g.gradient * s * inv_masses[g.index];
        }
    }
}

fn rebuild_spatial_hash(spatial_hash: &mut SpatialHash3D<Vec<(u32, usize)>>, mobjects: &mut [Mobject]) {
    spatial_hash.clear(&mut |v: &mut Vec<(u32, usize)>| v.clear());
    for mobj in mobjects.iter_mut() {
        mobj.update_spatial_hash(spatial_hash, mobj.id);
    }
}

fn generate_collision_constraints(
    cur_mobj_id: u32,
    new_mobj_positions: &[Vec3],
    spatial_hash: &SpatialHash3D<Vec<(u32, usize)>>,
    mobj_map: &HashMap<u32, &Mobject>,
) -> Vec<PhysicsConstraint> {
    let mut collisions: Vec<PhysicsConstraint> = Vec::new();
    for (j, v) in new_mobj_positions.iter().enumerate() {
        let pos = Vector3::new(v.x, v.y, v.z);
        spatial_hash
            .iter_cubes(pos, pos)
            .for_each(|(_, ent)| {
                ent.iter().for_each(|(mobj_id, i)| {
                    // NOTE: skipping self-collision
                    if *mobj_id == cur_mobj_id {
                        return ;
                    }
                    let m = mobj_map.get(mobj_id).unwrap();
                    let adj_vertices = m.get_physics_vertices();
                    let indices = m.indices();
                    let (v1, v2, v3) = (&adj_vertices[indices[*i * 3] as usize], &adj_vertices[indices[*i*3 + 1] as usize], &adj_vertices[indices[*i*3 + 2] as usize]);
                    let n = nalgebra_glm::cross(&(v2.position - v1.position), &(v3.position - v1.position));
                    let M = Matrix3x2::from_columns(&[v2.position - v1.position, v3.position - v1.position]);
                    let P = M * (M.transpose() * M).try_inverse().unwrap() * M.transpose();
                    let q_c = P * v;
                    collisions.push(
                        PhysicsConstraint::Collision(CollisionConstraint {
                            inp_vertex_index: j,
                            q: q_c,
                            n
                        }
                    ));
                })
            });
    }
    collisions
}

impl PBDSolver {
    pub fn update(
        &mut self,
        mobjects: &mut [Mobject],
        spatial_hash: &mut SpatialHash3D<Vec<(u32, usize)>>,
        dt: f32
    ) {
        if mobjects.len() == 0 {
            return;
        }
        for k in 0..mobjects.len() {
            // HACK: this isn't looking nice whatsoever, just a workaround for now...
            let (left, right) = mobjects.split_at_mut(k);
            let (mobj, rest) = right.split_first_mut().unwrap();
            let id = mobj.id;
            let (vertices, constraints) = mobj.get_mut_vertices_and_constraints();
            let mobj_map: HashMap<u32, &Mobject> = left.iter()
                .chain(rest.iter())
                .map(|m| (m.id, m))
                .collect();

            vertices.iter_mut().for_each(|v| {
                v.velocity += dt * v.w * Vec3::new(0.0, 0.0, -9.81);
            });

            let mut ps: Vec<Vec3> = vertices.iter().map(|v| v.position + dt * v.velocity).collect();
            // TODO: collision constraints
            let collisions = generate_collision_constraints(id, &ps, spatial_hash, &mobj_map);
            let mut constraints: Vec<PhysicsConstraint> = constraints.iter().map(|c| c.clone()).collect();
            constraints.extend(collisions.into_iter());

            let inv_masses: Vec<f32> = vertices.iter().map(|v| v.w).collect();
            let bs: Vec<Vec3> = vertices.iter().map(|v| v.body_space_position).collect();

            project_constraints(&mut ps, &bs, &inv_masses, &constraints);

            for (v, p) in vertices.iter_mut().zip(ps.iter()) {
                v.velocity = (p - v.position) / dt;
                v.position = *p;
            }
            let com = physics::center_of_mass(&vertices);
            mobj.set_com(com);
        }
        rebuild_spatial_hash(spatial_hash, mobjects);
    }
}
