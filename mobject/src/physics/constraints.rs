use nalgebra_glm::{Mat3, Vec3};
use crate::shapes::PhysicsVertex;
#[derive(Debug)]
pub struct ConstrainedGradient {
    pub index: usize,
    pub gradient: Vec3
}

pub trait Constraint {
    // C(p1, ..., pn)
    fn evaluate(&self, positions: &[Vec3]) -> f32;
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient>;
    // returns the denominator of the scaling factor, s
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

/// Maintain constant distance to `attached_to`
/// This was derived in Matthias' pbd paper
pub struct DistanceConstraint<'a> {
    pub inp_vertex_index: usize,
    pub attached_to: &'a PhysicsVertex,
    pub radius: f32
}

/// Constraint to make a mobject a rigid body
pub struct ShapeConstraint();

/// Implements C_stretch from https://matthias-research.github.io/pages/publications/posBasedDyn.pdf
pub struct StretchConstraint {
    pub vert_ind1: usize,
    pub vert_ind2: usize,
    pub l0: f32,
    // between 0-1
    pub k: f32
}

/// phi: the initial angle between two triangles
/// vert_ind{j}: the vertices in a pair of adjacent triangles (ind1, ind2 are common vertices in the triangle)
pub struct BendConstraint {
    pub vert_ind1: usize,
    pub vert_ind2: usize,
    pub vert_ind3: usize,
    pub vert_ind4: usize,
    pub k: f32,
    pub phi: f32
}

pub enum PhysicsConstraint {
    Attachment(AttachmentConstraint<'static>),
    Static(StaticConstraint),
    Distance(DistanceConstraint<'static>),
    Stretch(StretchConstraint),
    Bend(BendConstraint),
    RigidBody(ShapeConstraint)
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

impl Constraint for DistanceConstraint<'_> {
    fn evaluate(&self, positions: &[Vec3]) -> f32 {
        nalgebra_glm::l1_distance(&positions[self.inp_vertex_index], &self.attached_to.position) - self.radius
    }
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient> {
        let dist = nalgebra_glm::l1_distance(&positions[self.inp_vertex_index], &self.attached_to.position);

        vec![
            ConstrainedGradient {
                index: self.inp_vertex_index as usize,
                gradient: (positions[self.inp_vertex_index] - self.attached_to.position) / dist
            }
        ]
    }
}

impl Constraint for StretchConstraint {
    fn evaluate(&self, positions: &[Vec3]) -> f32 {
        // nalgebra_glm::l1_distance(&positions[self.vert_ind1], &positions[self.vert_ind2]) - self.l0
        let d = nalgebra_glm::distance(&positions[self.vert_ind1], &positions[self.vert_ind2]);
        d - self.l0
    }
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient> {
        let dist = nalgebra_glm::l1_distance(&positions[self.vert_ind1], &positions[self.vert_ind2]);
        // NOTE: downscaling by k
        vec![
            ConstrainedGradient {
                index: self.vert_ind1 as usize,
                gradient: self.k * (positions[self.vert_ind1] - positions[self.vert_ind2]) / dist
            },
            ConstrainedGradient {
                index: self.vert_ind2 as usize,
                gradient: self.k * (positions[self.vert_ind2] - positions[self.vert_ind1]) / dist
            }
        ]
    }
}

fn grad_n_wrt_p1(n: Vec3, p1: Vec3, p2: Vec3) -> Mat3 {
    let d = nalgebra_glm::cross(&p1, &p2).norm();
    // dbg!(p2, p2.cross_matrix());
    let m = - p2.cross_matrix() + nalgebra_glm::outer_product(&n, &nalgebra_glm::cross(&n, &p2));
    m * (1.0 / d)
}

impl Constraint for BendConstraint {
    fn evaluate(&self, positions: &[Vec3]) -> f32 {
    let p1 = positions[self.vert_ind1];
    let p2 = positions[self.vert_ind2] - p1;
    let p3 = positions[self.vert_ind3] - p1;
    let p4 = positions[self.vert_ind4] - p1;
    let n1 = nalgebra_glm::cross(&p2, &p3).normalize();
    let n2 = nalgebra_glm::cross(&p2, &p4).normalize();
    let d = n1.dot(&n2).clamp(-1.0, 1.0);
    d.acos() - self.phi
}

    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient> {
        // dbg!(positions[self.vert_ind1], positions[self.vert_ind2], positions[self.vert_ind3], positions[self.vert_ind4]);
        let p1 = positions[self.vert_ind1];
        let p2 = positions[self.vert_ind2] - p1;
        let p3 = positions[self.vert_ind3] - p1;
        let p4 = positions[self.vert_ind4] - p1;
        let n1 = nalgebra_glm::cross(&p2, &p3).normalize();
        let n2 = nalgebra_glm::cross(&p2, &p4).normalize();
        // let n1p3 = - grad_n_wrt_p1(n1, p3, p2).transpose();
        // let n2p4 = - grad_n_wrt_p1(n2, p4, p2).transpose();
        // let n1p2 = grad_n_wrt_p1(n1, p2, p3).transpose();
        // let n2p2 = grad_n_wrt_p1(n2, p2, p4).transpose();
        // dbg!(n1, n2);
        let d: f32 = nalgebra_glm::dot(&n1, &n2).clamp(-1.0 + 1e-3, 1.0 - 1e-3);
        // let c = - 1.0 / (1.0 - d * d).sqrt();
        // let grad_p3 = c * (n1p3 * n2);
        // let grad_p4 = c * (n2p4 * n1);
        // let grad_p2 = c * (n1p2 * n2 + n2p2 * n1);
        // let grad_p1 = - grad_p3 - grad_p4 - grad_p2;

        let p2n2 = nalgebra_glm::cross(&p2, &n2);
        let p2n1 = nalgebra_glm::cross(&p2, &n1);
        let p4n1 = nalgebra_glm::cross(&p4, &n1);
        let p3n2 = nalgebra_glm::cross(&p3, &n2);
        let p3n1 = nalgebra_glm::cross(&p3, &n1);
        let p4n2 = nalgebra_glm::cross(&p4, &n2);

        let p2p3_norm = nalgebra_glm::cross(&p2, &p3).norm();
        let p2p4_norm = nalgebra_glm::cross(&p2, &p4).norm();
        let q3 = (p2n2 - p2n1 * d) / p2p3_norm;
        let q4 = (p2n1 - p2n2 * d) / p2p4_norm;
        let q2 = -(p3n2-p3n1 * d) / p2p3_norm - (p4n1-p4n2 * d) / p2p4_norm;
        let q1 = - q2 - q3 - q4;
        // dbg!(d, n1, n2, q4, q3, q2, q1);
        vec![
            ConstrainedGradient {
                index: self.vert_ind1 as usize,
                gradient: self.k * q1
            },
            ConstrainedGradient {
                index: self.vert_ind2 as usize,
                gradient: self.k * q2
            },
            ConstrainedGradient {
                index: self.vert_ind3 as usize,
                gradient: self.k * q3
            },
            ConstrainedGradient {
                index: self.vert_ind4 as usize,
                gradient: self.k *  q4
            }
        ]
    }
}

impl Constraint for PhysicsConstraint {
    fn evaluate(&self, positions: &[Vec3]) -> f32 {
        match self {
            PhysicsConstraint::Static(c) => c.evaluate(positions),
            PhysicsConstraint::Distance(c) => c.evaluate(positions),
            PhysicsConstraint::Attachment(c) => c.evaluate(positions),
            PhysicsConstraint::Stretch(c) => c.evaluate(positions),
            PhysicsConstraint::Bend(c) => c.evaluate(positions),
            PhysicsConstraint::RigidBody(_) => panic!("Rigid body constraints do not have an evaluate function")
        }
    }
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient> {
        match self {
            PhysicsConstraint::Static(c) => c.gradient(positions),
            PhysicsConstraint::Distance(c) => c.gradient(positions),
            PhysicsConstraint::Attachment(c) => c.gradient(positions),
            PhysicsConstraint::Stretch(c) => c.gradient(positions),
            PhysicsConstraint::Bend(c) => c.gradient(positions),
            PhysicsConstraint::RigidBody(_) => panic!("Rigid body constraints do not have a gradient function")
        }
    }
}
