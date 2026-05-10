pub mod surface;
pub mod primitives;
pub mod generator;
pub mod animations;
pub mod objects;
pub use primitives::*;
pub use objects::*;
pub use animations::*;

pub use surface::Points;
use ash::Device;
use ash::vk::{
    self, Buffer, DeviceMemory, PhysicalDeviceMemoryProperties, VertexInputAttributeDescription,
    VertexInputBindingDescription,
};
use nalgebra_glm as glm;
use nalgebra_glm::{Vec2, Vec3};
use std::fmt::Debug;
use std::{mem::offset_of, os::raw::c_void};
use crate::physics::constraints::{PhysicsConstraint, ShapeConstraint, StaticConstraint};
use crate::pipelines::Pipelines;
use crate::scene::Mobject;
use crate::{MAX_FRAMES_IN_FLIGHT};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct UBO {
    pub model: glm::Mat4,
}

#[repr(C)]
#[derive(Debug)]
pub struct GlobalUBO {
    pub camera_position: glm::Vec3,
    pub _pad: u32, // 4 bytes of padding, because std140 rule states vec3 takes 16 bytes...
    pub view: glm::Mat4,
    pub proj: glm::Mat4,
    pub light_space: glm::Mat4,
}

pub trait Vertex<const N: usize> {
    fn attribute_descriptions() -> [VertexInputAttributeDescription; N];
    fn binding_description() -> VertexInputBindingDescription;
}

#[derive(Clone, Copy)]
pub enum Manifold {
    TwoD,
    ThreeD,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RenderVertex {
    pub position: Vec3,
    pub color: Vec3,
    pub normal: Vec3,
    pub tex_coord: Vec2,
}
#[derive(Debug)]
pub struct PhysicsVertex {
    pub position: Vec3,
    pub velocity: Vec3,
    // inverse of mass: 1/m
    pub w: f32,
    // relevant for rigid bodies only. depends on obj's COM to get
    // world space.
    pub body_space_position: Vec3,
    pub mobject_id: u32,
}

/// A wrapper during construction of a mobject:
/// tells the Scene what the intent of the mobject is, such as
/// whether it is rigid, which vertices are anchored, etc.
pub struct ShapeIntent {
    pub entity: Box<dyn Shape>,
    pub constraints: Vec<PhysicsConstraint>,
    pub inverse_masses: Vec<f32>,
    pub velocities: Vec<Vec3>,
}

impl ShapeIntent {
    pub fn new(entity: Box<dyn Shape>) -> Self {
        let s = entity.get_vertices().len();
        Self {
            entity,
            constraints: Vec::new(),
            inverse_masses: vec![1.0; s],
            velocities: vec![Vec3::new(0.0, 0.0, 0.0); s],
        }
    }
    pub fn anchor(mut self, anchors: Vec<usize>) -> Self {
        let verts = self.entity.get_vertices();
        for i in anchors {
            // NOTE: scene will fill pin_to
            self.constraints.push(PhysicsConstraint::Static(StaticConstraint { inp_vertex_index: i, pin_to: verts[i].position }));
            self.inverse_masses[i] = 0.0;
        }
        self
    }
    pub fn make_rigid(mut self) -> Self {
        self.constraints.push(PhysicsConstraint::RigidBody(ShapeConstraint { start_physics_vertices_idx: 0, total_physics_vertices: self.entity.get_vertices().len() }));
        self
    }
    pub fn with_constraints(mut self, constraints: Vec<PhysicsConstraint>) -> Self {
        self.constraints = constraints;
        self
    }
    pub fn make_static(self) -> Self {
        let s = self.entity.get_vertices().len();
        self.anchor((0..s).collect())
    }
    pub fn with_velocities(mut self, velocities: Vec<Vec3>) -> Self {
        self.velocities = velocities;
        self
    }
    pub fn with_inverse_masses(mut self, inverse_masses: Vec<f32>) -> Self {
        self.inverse_masses = inverse_masses;
        self
    }
}

impl PhysicsVertex {
    pub fn new(position: Vec3, mobj_id: u32) -> Self {
        Self {
            position,
            velocity: Vec3::new(0.0, 0.0, 0.0),
            w: 1.0,
            body_space_position: Vec3::new(0.0, 0.0, 0.0),
            mobject_id: mobj_id
        }
    }
    pub fn with_body_space_position(&mut self, body_position: Vec3) {
        self.body_space_position = body_position;
    }
    pub fn set_velocity(&mut self, velocity: Vec3) {
        self.velocity = velocity;
    }
    pub fn set_inv_mass(&mut self, inv_mass: f32) {
        self.w = inv_mass;
    }
}

pub fn compute_normals(indices: &[usize], vertices_position: &[Vec3]) -> Vec<Vec3> {
    assert!(!vertices_position.is_empty());
    assert!(indices.len() % 3 == 0);
    let mut normals = vec![Vec3::zeros(); vertices_position.len()];
    for idxes in indices.chunks_exact(3) {
        let [a, b, c] = [idxes[0], idxes[1], idxes[2]];
        let p0 = vertices_position[a];
        let p1 = vertices_position[b];
        let p2 = vertices_position[c];
        let v1 = p1 - p0;
        let v2 = p2 - p0;
        let face_normal = glm::cross(&v1, &v2);
        normals[a] += face_normal;
        normals[b] += face_normal;
        normals[c] += face_normal;
    }
    for n in &mut normals {
        *n = glm::normalize(n);
    }
    normals
}

impl RenderVertex {
    pub fn new(position: Vec3, color: Vec3, normal: Vec3, tex_coord: Option<Vec2>) -> Self {
        Self {
            position,
            color,
            normal,
            tex_coord: tex_coord.unwrap_or(Vec2::new(0.0, 0.0)),
        }
    }
    pub fn with_tex_coord(position: Vec3, normal: Vec3, tex_coord: Vec2) -> Self {
        Self {
            position,
            normal,
            color: Vec3::new(1.0, 1.0, 1.0),
            tex_coord,
        }
    }
    pub fn update_tex_coord(self, c: Vec2) -> Self {
        Self {
            position: self.position,
            color: self.color,
            normal: self.normal,
            tex_coord: c,
        }
    }
}

impl Vertex<4> for RenderVertex {
    fn binding_description() -> VertexInputBindingDescription {
        VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<RenderVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    fn attribute_descriptions() -> [VertexInputAttributeDescription; 4] {
        [
            VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(RenderVertex, position) as u32,
            },
            VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(RenderVertex, color) as u32,
            },
            VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(RenderVertex, normal) as u32,
            },
            VertexInputAttributeDescription {
                binding: 0,
                location: 3,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(RenderVertex, tex_coord) as u32
            }
        ]
    }
}

pub trait ShapeMotion {
    fn get_ubo_contents(&self) -> &UBO;
    fn rotate(&mut self, axis: &glm::Vec3, angle: f32);
}

pub trait ShapeConstruction {
    fn get_pipeline(&self) -> Pipelines;
    fn vertices2d(&self) -> &[RenderVertex] {
        self.get_vertices()
    }
    fn get_vertices(&self) -> &[RenderVertex];
    fn indices(&self) -> &[u32];
    fn vertices_and_indices(&self) -> (&[RenderVertex], &[u32]) {
        (self.get_vertices(), self.indices())
    }
    fn get_uniform_buffer(
        &self,
    ) -> (
        &[Buffer; MAX_FRAMES_IN_FLIGHT as usize],
        &[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize],
        &[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize],
    );
    fn texture_path(&self) -> Option<&str>;
    fn sync_phys_and_render_vertices(&mut self, physics_vertices: &[PhysicsVertex]);
    fn set_com(&mut self, com: Vec3);
    fn update_spatial_hash(&self, space: &mut shapeject::SpatialHash3D<Vec<(u32, usize)>>, mid: u32);
    fn set_physics_index_range(&mut self, start: usize, total: usize);
    fn get_physics_index_range(&self) -> (usize, usize);
    fn manifold(&self) -> Manifold;
}

pub trait BuiltShape: ShapeMotion + ShapeConstruction {}

pub trait Shape {
    fn build(
        self: Box<Self>,
        device: &Device,
        mem_properties: PhysicalDeviceMemoryProperties,
    ) -> Box<dyn BuiltShape>;
    fn texture_path(&self) -> Option<&str>;
    fn get_vertices(&self) -> &[RenderVertex];
}

pub fn mobjects_to_vertices_and_indices(
    mobjects: &[&Mobject],
) -> (Vec<RenderVertex>, Vec<u32>) {
    let mut vertices: Vec<RenderVertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..mobjects.len()  {
        let current_length = vertices.len() as u32;
        let shape_vertices = mobjects[i].vertices2d();
        let shape_indices = mobjects[i].indices();
        shape_vertices.iter().for_each(|f| vertices.push(f.clone()));
        shape_indices
            .iter()
            .for_each(|&i| indices.push(i + current_length));
    }
    (vertices, indices)
}
