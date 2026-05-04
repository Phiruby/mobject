use std::collections::HashMap;

use cgmath::Vector3;
use nalgebra_glm::{Vec3, Mat3};
use shapeject::SpatialHash3D;
use spatial_hash_3d::SpatialHashGrid;
use crate::physics::constraints::{self, CollisionConstraint, Constraint, Equality, PhysicsConstraint};
use crate::shapes::{Manifold, PhysicsVertex};
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
        // TODO: hacking over rigid constraint
        if let PhysicsConstraint::RigidBody(x) = c {
            let start = x.start_physics_vertices_idx;
            let end = start + x.total_physics_vertices;

            let wp = &mut world_positions[start..end];
            let inv_mass: &[f32] = &inv_masses[start..end];
            let current_com = physics::raw_center_of_mass(wp, inv_mass);
            let bp: &[Vec3] = &body_positions[start..end];
            shape_matching(wp, current_com, bp, inv_mass);
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
        let k = c.k();
        let denom = grads.iter().fold(0.0, |acc, grad| acc + inv_masses[grad.index] * grad.gradient.norm_squared());
        let s = ev / denom;
        for g in grads.iter() {
            world_positions[g.index] -= k * g.gradient * s * inv_masses[g.index];
        }
    }
}

pub fn rebuild_spatial_hash(spatial_hash: &mut SpatialHash3D<Vec<(u32, usize)>>, mobjects: &mut [Mobject]) {
    spatial_hash.clear(&mut |v: &mut Vec<(u32, usize)>| v.clear());
    for mobj in mobjects.iter_mut() {
        mobj.update_spatial_hash(spatial_hash, mobj.id);
    }
}

fn generate_collision_constraints(
    mobjects: &[Mobject],
    physics_vertices: &[PhysicsVertex],
    new_mobj_positions: &[Vec3],
    spatial_hash: &SpatialHash3D<Vec<(u32, usize)>>,
) -> Vec<PhysicsConstraint> {
    let mut bee = false;
    let mut collisions: Vec<PhysicsConstraint> = Vec::new();
    for (j, (v, old_v)) in new_mobj_positions
        .iter()
        .zip(physics_vertices.iter())
        .enumerate() {
        if old_v.w <= 0.0 { continue ; }
        let pos = Vector3::new(v.x, v.y, v.z);
        spatial_hash
            .iter_cubes(pos, pos)
            .for_each(|(_, ent)| {
                ent.iter().for_each(|(mobj_id, i)| {
                    // NOTE: skipping self-collision
                    if *mobj_id == old_v.mobject_id {
                        return ;
                    }
                    let m: &Mobject = &mobjects[*mobj_id as usize];
                    let (start, total) = m.get_physics_index_range();
                    let adj_vertices = &physics_vertices[start..start + total];
                    let indices = m.indices();
                    let (v1, v2, v3) = (&adj_vertices[indices[*i * 3] as usize], &adj_vertices[indices[*i*3 + 1] as usize], &adj_vertices[indices[*i*3 + 2] as usize]);
                    let e1 = v2.position - v1.position;
                    let e2 = v3.position - v1.position;
                    let mut n = nalgebra_glm::cross(&e1, &e2);
                    // TODO: maybe don't need this check?
                    n = n.normalize();

                    let v_to_p = v - v1.position;
                    let dist = nalgebra_glm::dot(&v_to_p, &n);
                    let q_c = v - (dist * n);
                    let old_ray: Vec3 = old_v.position - q_c;
                    let new_ray = v - q_c;
                    if matches!(m.manifold(), Manifold::TwoD) && old_ray.dot(&n) < 0.0 {
                        n = -n;
                    }
                    // if old and new in same direction relative to point of contact, skip and continue
                    if old_ray.dot(&n) * new_ray.dot(&n) > 0.0 {
                        return;
                    }
                    // 2D objects (e.g rectangle) will need some thickness so it doesn't stick on the surface of the collidee
                    // so we add thickness to the collision constraint
                    let mut thickness = 0.0;
                    if matches!(mobjects[old_v.mobject_id as usize].manifold(), Manifold::TwoD) {
                        thickness = 0.0001;
                    }
                    collisions.push(
                        PhysicsConstraint::Collision(CollisionConstraint {
                            inp_vertex_index: j,
                            q: q_c,
                            n: n,
                            thickness
                        }
                    ));
                    // }
                })
            });
    }
    collisions
}

fn dampen_velocities(
    scene_vertices: &mut [PhysicsVertex],
    mobjects: &[Mobject],
    k_damping: Option<f32>
) {
    for m in mobjects.iter() {
        let (start, total) = m.get_physics_index_range();
        let vertices = &mut scene_vertices[start..start + total];
        // if all vertices are static, return to avoid inverting I
        if vertices.iter().all(|x| x.w <= 0.0) { return; }
        let k_damping = k_damping.unwrap_or(0.3);
        let (x_cm, v_cm) = physics::center_of_mass(&vertices);
        let L = vertices
            .iter()
            // NOTE: infinite mass => velocity 0 anyways
            .filter(|x| x.w > 0.0)
            .fold(Vec3::new(0.0, 0.0, 0.0), |acc, x| acc + nalgebra_glm::cross(&(x.position - x_cm),  &x.velocity) * (1.0 / x.w));
        let I = vertices
            .iter()
            .filter(|x| x.w > 0.0)
            .fold(Mat3::zeros(), |acc, x| acc + (1.0 / x.w) * nalgebra_glm::matrix_cross3(&(x.position - x_cm)) * nalgebra_glm::matrix_cross3(&(x.position - x_cm)).transpose());
        let omega = I.try_inverse().unwrap() * L;
        for v in vertices.iter_mut() {
            if v.w <= 0.0 { continue; }
            let dv = v_cm - nalgebra_glm::cross(&(v.position - x_cm), &omega) - v.velocity;
            v.velocity = v.velocity + k_damping * dv;
        }
    }
}

impl PBDSolver {
    pub fn update(
        &mut self,
        mobjects: &[Mobject],
        physics_vertices: &mut [PhysicsVertex],
        constraints: &[PhysicsConstraint],
        spatial_hash: &mut SpatialHash3D<Vec<(u32, usize)>>,
        dt: f32
    ) {
        if mobjects.len() == 0 {
            return;
        }
        physics_vertices
            .iter_mut()
            .for_each(|v| {
                v.velocity += dt * v.w * Vec3::new(0.0, 0.0, -9.81);
            });
        dampen_velocities(physics_vertices, mobjects, None);
        let mut ps: Vec<Vec3> = physics_vertices.iter().map(|v| v.position + dt * v.velocity).collect();
        let mut constraints: Vec<PhysicsConstraint> = constraints.iter().map(|c| c.clone()).collect();
        // NOTE: collision constraints affect attachment constraints, so need to fix
        let collision_constraints = generate_collision_constraints(
            mobjects,
            physics_vertices,
            &ps,
            spatial_hash,
        );
        constraints.extend(collision_constraints.into_iter());
        let inv_masses: Vec<f32> = physics_vertices.iter().map(|v| v.w).collect();
        let bs: Vec<Vec3> = physics_vertices.iter().map(|v| v.body_space_position).collect();
        // TODO: project_constraints computes x_cm using all vertices, which is wrong; let's fix this
        for _ in 0..15 {
            project_constraints(&mut ps, &bs, &inv_masses, &constraints);
        }
        for (v, p) in physics_vertices.iter_mut().zip(ps.iter()) {
            v.velocity = (p - v.position) / dt;
            v.position = *p;
        }
    }
}
