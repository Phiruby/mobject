use std::collections::HashMap;

use nalgebra_glm::{Mat3, Vec3};
use crate::{scene::Mobject, shapes::PhysicsVertex};


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

/// Represents a constraint on vertices *within* a single mobject
/// (for example, anything that implements this trait will not be able to
/// apply constraints between two different mobjects)
pub trait Constraint {
    // C(p1, ..., pn)
    fn evaluate(&self, positions: &[Vec3]) -> f32;
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient>;
}

/// Attach to a particular vertex.
/// `inp_vertex_index` represents the index of the vertex of parent `mobj` that you want to attach
/// the `attach_to` vertex to
#[derive(Clone, Debug)]
pub struct AttachmentConstraint {
    pub left_vertex_index: usize,
    pub right_vertex_index: usize,
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
pub struct ShapeConstraint {
    pub start_physics_vertices_idx: usize,
    pub total_physics_vertices: usize
}

// NOTE: this can be used for both continuous and static collisions
// provided that the right arguments are passed (it is on the caller to compute the correct)
// q and n, which represent point of contact and normal, respectively
#[derive(Clone, Debug)]
pub struct CollisionConstraint {
    pub inp_vertex_index: usize,
    pub q: Vec3,
    pub n: Vec3,
    pub thickness: f32
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
    Attachment(AttachmentConstraint),
    Static(StaticConstraint),
    Stretch(StretchConstraint),
    RigidBody(ShapeConstraint),
    Collision(CollisionConstraint)
}


impl Constraint for AttachmentConstraint {
    fn evaluate(&self, positions: &[Vec3]) -> f32 {
        let ml = positions[self.left_vertex_index];
        let mr = positions[self.right_vertex_index];
        (ml - mr).norm()
    }
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient> {
    let diff = positions[self.left_vertex_index] - positions[self.right_vertex_index];
    let dist = diff.norm();
    if dist < 1e-8 {
        return vec![];
    }
    let dir = diff / dist;
    vec![
        ConstrainedGradient { index: self.left_vertex_index,  gradient: dir  },
        ConstrainedGradient { index: self.right_vertex_index, gradient: -dir },
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
        (positions[self.inp_vertex_index] - self.q).dot(&self.n) - self.thickness
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

    pub fn add_offset(&mut self, offset: usize) {
        match self {
            PhysicsConstraint::Static(i) => i.inp_vertex_index += offset,
            PhysicsConstraint::Stretch(i) => {i.vert_ind1 += offset; i.vert_ind2 += offset},
            PhysicsConstraint::Collision(i) => i.inp_vertex_index += offset,
            PhysicsConstraint::RigidBody(i) => { i.start_physics_vertices_idx += offset;},
            PhysicsConstraint::Attachment(_) => panic!("Attachment constraints depend on multiple mobjects. Cannot add offset"),
        }
    }

    pub fn k(&self) -> f32 {
        match self {
            PhysicsConstraint::Stretch(s) => s.k,
            // PhysicsConstraint::Collision(_) => 3.0,
            _ => 1.0
        }
    }
}
