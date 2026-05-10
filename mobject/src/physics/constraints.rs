
use nalgebra_glm::Vec3;
use crate::physics;


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
    pub triangle_indices: [usize; 3],
    pub n: Vec3,
    pub thickness: f32,
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
        let p = positions[self.inp_vertex_index];
        let v1 = positions[self.triangle_indices[0]];
        let v2 = positions[self.triangle_indices[1]];
        let v3 = positions[self.triangle_indices[2]];
        let v_to_p = p - v1;
        let dist = v_to_p.dot(&self.n);
        let q_c = p - (dist * self.n);

        if !physics::barycentric_test(&q_c, &v1, &v2, &v3) {
            return 0.0;
        }

        dist - self.thickness
    }

    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient> {
        let p = positions[self.inp_vertex_index];
        let v1 = positions[self.triangle_indices[0]];
        let v2 = positions[self.triangle_indices[1]];
        let v3 = positions[self.triangle_indices[2]];

        let v_to_p = p - v1;
        let dist = v_to_p.dot(&self.n);
        let q_c = p - (dist * self.n);
        let (w1, w2, w3) = physics::get_barycentric_coords(&q_c, &v1, &v2, &v3);

        vec![
            ConstrainedGradient { index: self.inp_vertex_index, gradient: self.n },
            ConstrainedGradient { index: self.triangle_indices[0], gradient: -self.n * w1 },
            ConstrainedGradient { index: self.triangle_indices[1], gradient: -self.n * w2 },
            ConstrainedGradient { index: self.triangle_indices[2], gradient: -self.n * w3 },
        ]
    }
}

impl Constraint for StretchConstraint {
    fn evaluate(&self, positions: &[Vec3]) -> f32 {
        let d = nalgebra_glm::distance(&positions[self.vert_ind1], &positions[self.vert_ind2]);
        d - self.l0
    }
    fn gradient(&self, positions: &[Vec3]) -> Vec<ConstrainedGradient> {
        let a = positions[self.vert_ind1];
        let b = positions[self.vert_ind2];
        let diff = a - b;
        let dist = nalgebra_glm::length(&diff).max(1e-4);
        vec![
            ConstrainedGradient {
                index: self.vert_ind1 as usize,
                gradient: diff / dist
            },
            ConstrainedGradient {
                index: self.vert_ind2 as usize,
                gradient: -diff / dist
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
