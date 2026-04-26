use nalgebra_glm::{Vec3};
use crate::physics::constraints::{Constraint};
use crate::{scene::Mobject};
pub struct PBDSolver ();

fn project_constraints(
    positions: &mut [Vec3],
    inv_masses: &[f32],
    constraints: &[&dyn Constraint],
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
            let vertices = mobj.get_mut_vertices();

            vertices.iter_mut().for_each(|v| {
                v.velocity += dt * v.w * Vec3::new(0.0, 0.0, -9.81);
            });

            let mut ps: Vec<Vec3> = vertices.iter().map(|v| v.position + dt * v.velocity).collect();
            // TODO: collision constraints
            let inv_masses: Vec<f32> = vertices.iter().map(|v| v.w).collect();
            let constraints: Vec<&dyn Constraint> = vertices
                .iter()
                .flat_map(|v| v.constraints.iter())
                .map(|c| c.as_ref())
                .collect();

            project_constraints(&mut ps, &inv_masses, &constraints);
            for (v, p) in vertices.iter_mut().zip(ps.iter()) {
                v.velocity = (p - v.position) / dt;
                v.position = *p;
            }
        }
    }
}
