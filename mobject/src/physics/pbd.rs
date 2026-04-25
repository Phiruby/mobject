use nalgebra_glm::{Mat3x3, Vec3, Quat};

use crate::shapes::{BuiltShape, Vertex2D};
struct PBDSolver ();

struct ConstrainedGradient {
    index: usize,
    gradient: Vec3
}

pub trait Constraint {
    // C(p1, ..., pn)
    fn evaluate(&self, positions: &[Vec3]) -> f32;
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient>;
    // returns the denominator of the scaling factor, s
    fn scale_factor(&self, positions: &[Vec3], inv_masses: &[f32]) -> f32 {
        let grads = self.gradient(positions);
        grads.iter().fold(0.0, |acc, grad| acc + inv_masses[grad.index] * grad.gradient.norm())
    }
}

/// Attach to a particular vertex.
/// `inp_vertex_index` represents the index of the vertex of parent `mobj` that you want to attach
/// the `attach_to` vertex to
pub struct AttachmentConstraint<'a> {
    inp_vertex_index: usize,
    attached_to: &'a Vertex2D
}

impl Constraint for AttachmentConstraint<'_> {
    fn evaluate(&self, positions: &[Vec3]) -> f32 {
        (positions[self.inp_vertex_index]- self.attached_to.position).norm()
    }
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient> {
        vec![
            ConstrainedGradient {
                index: self.inp_vertex_index as usize,
                gradient: (positions[self.inp_vertex_index] - self.attached_to.position) * 2.0
            }
        ]
    }
}

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
    pub fn update(&mut self, mobjects: &mut [Box<dyn BuiltShape>], dt: f32) {
        // TODO: external forces
        for mobj in mobjects {
            let vertices = mobj.get_mut_vertices();
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
