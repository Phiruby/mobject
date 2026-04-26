use nalgebra_glm::{Vec3};
use crate::shapes::PhysicsVertex;
pub struct ConstrainedGradient {
    pub index: usize,
    pub gradient: Vec3
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
    pub inp_vertex_index: usize,
    pub attached_to: &'a PhysicsVertex
}

/// Pins to a particular point.
/// Unlike `AttachmentConstraint`, the point this is pinned to
/// is a static vector: it does not change
pub struct StaticConstraint {
    pub inp_vertex_index: usize,
    pub pin_to: Vec3
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

impl Constraint for StaticConstraint {
    fn evaluate(&self, positions: &[Vec3]) -> f32 {
        (positions[self.inp_vertex_index] - self.pin_to).norm()
    }
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient> {
        vec![
            ConstrainedGradient {
                index: self.inp_vertex_index as usize,
                gradient: (positions[self.inp_vertex_index] - self.pin_to) * 2.0
            }
        ]
    }
}
