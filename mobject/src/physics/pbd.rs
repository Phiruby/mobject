use nalgebra_glm::{Mat3x3, Vec3, Quat};

use crate::shapes::{BuiltShape, Vertex2D};
struct OdeSolver ();

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
    inp_vertex_index: i32,
    // 0, 1, 2 -> x, y, z
    coordinate: usize,
    attached_to: &'a Vertex2D
}


impl OdeSolver {
    fn solve(&self, dt: f32) {
        // TODO: main PDB loop
    }
}
