use nalgebra_glm::{Vec3};
use crate::physics::constraints::{Constraint, PhysicsConstraint};
use crate::{scene::Mobject};
pub struct PBDSolver ();

fn project_constraints(
    positions: &mut [Vec3],
    inv_masses: &[f32],
    constraints: &[PhysicsConstraint],
) {
    for c in constraints {
        let ev = c.evaluate(positions);
        if ev.abs() <= f32::EPSILON {
            continue;
        }
        let denom = c.scale_factor(positions, inv_masses);
        let s = ev / denom;
        let grads = c.gradient(positions);
        for g in grads {
            positions[g.index] -= g.gradient * s * inv_masses[g.index];
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

            project_constraints(&mut ps, &inv_masses,constraints);
            for (v, p) in vertices.iter_mut().zip(ps.iter()) {
                v.velocity = (p - v.position) / dt;
                v.position = *p;
            }
        }
    }
}
