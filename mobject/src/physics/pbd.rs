use nalgebra_glm::{Vec3, Mat3};
use crate::physics::constraints::{Constraint, PhysicsConstraint};
use crate::{scene::Mobject};
use crate::physics;
use nalgebra::{Complex, SVD};
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
        // .filter(|((_, _), m)| **m > 0.0)
        .map(
            |((w, b), &m)| nalgebra_glm::outer_product(&(w - current_com), b)
        )
        .sum::<nalgebra_glm::Mat3>();
    // TODO: can precompute this at the beginning
    let m2 = body_positions
        .iter()
        .zip(inv_masses.iter())
        // .filter(|(_, m)| **m > 0.0)
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
        if ev.abs() <= f32::EPSILON {
            continue;
        }
        let grads = c.gradient(world_positions);
        let denom = grads.iter().fold(0.0, |acc, grad| acc + inv_masses[grad.index] * grad.gradient.norm());
        let s = ev / denom;
        for g in grads {
            world_positions[g.index] -= g.gradient * s * inv_masses[g.index];
        }
    }
}

impl PBDSolver {
    pub fn update(&mut self, mobjects: &mut [Mobject], dt: f32) {
        // TODO: external forces
        if mobjects.len() == 0 {
            return;
        }
        for mobj in mobjects {
            let (vertices, constraints) = mobj.get_mut_vertices_and_constraints();

            vertices.iter_mut().for_each(|v| {
                v.velocity += dt * v.w * Vec3::new(0.0, 0.0, -9.81);
            });

            let mut ps: Vec<Vec3> = vertices.iter().map(|v| v.position + dt * v.velocity).collect();
            // TODO: collision constraints
            let inv_masses: Vec<f32> = vertices.iter().map(|v| v.w).collect();
            let bs: Vec<Vec3> = vertices.iter().map(|v| v.body_space_position).collect();
            project_constraints(&mut ps, &bs, &inv_masses,constraints);
            for (v, p) in vertices.iter_mut().zip(ps.iter()) {
                v.velocity = (p - v.position) / dt;
                v.position = *p;
            }
            let com = physics::center_of_mass(&vertices);
            mobj.set_com(com);
        }
    }
}
