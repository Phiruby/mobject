pub mod pbd;
pub mod constraints;
use nalgebra_glm::{Vec3};
use crate::shapes::PhysicsVertex;

/// Computes center of mass normally:
/// If any vertices have infinite mass, we use a (uniform) weighted
/// average of infinite mass positions. Otherwise, we compute CoM as usual
pub fn center_of_mass(vertices: &[PhysicsVertex]) -> Vec3 {
    let inf_masses=  vertices.iter().filter(|x| x.w == 0.0);
    // TODO: cloning just for count!
    let c = inf_masses.clone().count();
    if c > 0 {
        return inf_masses.fold(Vec3::new(0.0, 0.0, 0.0), |acc, x| acc + x.position) / c as f32;
    }
    let mut com = Vec3::new(0.0, 0.0, 0.0);
    let mut total_mass = 0.0;
    for v in vertices {
        com += v.position / v.w;
        total_mass += 1.0 / v.w;
    }
    com /= total_mass;
    com
}

pub fn raw_center_of_mass(vertices: &[Vec3], inv_masses: &[f32]) -> Vec3 {
    let inf_masses = vertices
        .iter()
        .zip(inv_masses.iter())
        .filter(|(_, m)| **m == 0.0);
    let c = inf_masses.clone().count();
    if c > 0 {
        return inf_masses.fold(Vec3::new(0.0, 0.0, 0.0), |acc, (x, _)| acc + x) / c as f32;
    }
    let mut com = Vec3::new(0.0, 0.0, 0.0);
    let mut total_mass = 0.0;
    for (v, m) in vertices.iter().zip(inv_masses.iter()) {
        com += *v * 1.0 / *m;
        total_mass += 1.0 / *m;
    }
    com /= total_mass;
    com
}
