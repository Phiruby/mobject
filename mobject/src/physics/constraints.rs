use nalgebra_glm::{Mat3, Vec3};
use crate::shapes::PhysicsVertex;


#[derive(Debug)]
pub struct ConstrainedGradient {
    pub index: usize,
    pub gradient: Vec3
}

#[derive(PartialEq)]
pub enum Equality {
    Equal,
    Inequality
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
#[derive(Clone, Debug)]
pub struct AttachmentConstraint<'a> {
    pub inp_vertex_index: usize,
    pub attached_to: &'a PhysicsVertex
}

/// Pins to a particular point.
/// Unlike `AttachmentConstraint`, the point this is pinned to
/// is a static vector: it does not change
#[derive(Clone, Debug)]
pub struct StaticConstraint {
    pub inp_vertex_index: usize,
    pub pin_to: Vec3
}

/// Constraint to make a mobject a rigid body
#[derive(Clone, Debug)]
pub struct ShapeConstraint();

// NOTE: this can be used for both continuous and static collisions
// provided that the right arguments are passed (it is on the caller to compute the correct)
// q and n, which represent point of contact and normal, respectively
#[derive(Clone, Debug)]
pub struct CollisionConstraint {
    pub inp_vertex_index: usize,
    pub q: Vec3,
    pub n: Vec3
}

/// Implements C_stretch from https://matthias-research.github.io/pages/publications/posBasedDyn.pdf
#[derive(Clone, Debug)]
pub struct StretchConstraint {
    pub vert_ind1: usize,
    pub vert_ind2: usize,
    pub l0: f32,
    // between 0-1
    pub k: f32
}

#[derive(Clone, Debug)]
pub enum PhysicsConstraint {
    Attachment(AttachmentConstraint<'static>),
    Static(StaticConstraint),
    Stretch(StretchConstraint),
    RigidBody(ShapeConstraint),
    Collision(CollisionConstraint)
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

impl Constraint for CollisionConstraint {
    fn evaluate(&self, positions: &[Vec3]) -> f32 {
        (positions[self.inp_vertex_index] - self.q).dot(&self.n) - 0.01
    }
    fn gradient(&self, _: &[Vec3]) -> Vec<ConstrainedGradient> {
        vec![
            ConstrainedGradient {
                index: self.inp_vertex_index as usize,
                gradient: self.n
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

impl Constraint for PhysicsConstraint {
    fn evaluate(&self, positions: &[Vec3]) -> f32 {
        match self {
            PhysicsConstraint::Static(c) => c.evaluate(positions),
            PhysicsConstraint::Attachment(c) => c.evaluate(positions),
            PhysicsConstraint::Stretch(c) => c.evaluate(positions),
            PhysicsConstraint::Collision(c) => c.evaluate(positions),
            PhysicsConstraint::RigidBody(_) => panic!("Rigid body constraints do not have an evaluate function")
        }
    }
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient> {
        match self {
            PhysicsConstraint::Static(c) => c.gradient(positions),
            PhysicsConstraint::Attachment(c) => c.gradient(positions),
            PhysicsConstraint::Stretch(c) => c.gradient(positions),
            PhysicsConstraint::Collision(c) => c.gradient(positions),
            PhysicsConstraint::RigidBody(_) => panic!("Rigid body constraints do not have a gradient function")
        }
    }
}

impl PhysicsConstraint {
    pub fn equality_type(&self) -> Equality {
        match self {
            PhysicsConstraint::Static(_) => Equality::Equal,
            PhysicsConstraint::Attachment(_) => Equality::Equal,
            PhysicsConstraint::Stretch(_) => Equality::Equal,
            PhysicsConstraint::Collision(_) => Equality::Inequality,
            PhysicsConstraint::RigidBody(_) => panic!("Rigid body constraints do not have an equality type")
        }
     }
}
