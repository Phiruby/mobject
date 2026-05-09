pub mod pbd;
pub mod constraints;
use nalgebra_glm::{Vec3};
use crate::shapes::PhysicsVertex;

/// Computes center of mass normally:
/// If any vertices have infinite mass, we use a (uniform) weighted
/// average of infinite mass positions. Otherwise, we compute CoM as usual.
/// Returns x_cm and v_cm
pub fn center_of_mass(vertices: &[PhysicsVertex]) -> (Vec3, Vec3) {
    let inf_masses: Vec<&PhysicsVertex> =  vertices.iter().filter(|x| x.w == 0.0).collect();
    // TODO: cloning just for count!
    let c = inf_masses.len();
    if c > 0 {
        let x_cm = inf_masses.iter().fold(Vec3::new(0.0, 0.0, 0.0), |acc, x| acc + x.position) / c as f32;
        let v_cm = inf_masses.iter().fold(Vec3::new(0.0, 0.0, 0.0), |acc, x| acc + x.velocity) / c as f32;
        return (x_cm, v_cm);
    }
    let mut com = Vec3::new(0.0, 0.0, 0.0);
    let mut v_com = Vec3::new(0.0, 0.0, 0.0);
    let mut total_mass = 0.0;
    for v in vertices {
        com += v.position / v.w;
        v_com += v.velocity / v.w;
        total_mass += 1.0 / v.w;
    }
    com /= total_mass;
    v_com /= total_mass;
    (com, v_com)
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

pub fn get_barycentric_coords(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3) -> (f32, f32, f32) {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = nalgebra_glm::dot(&v0, &v0);
    let d01 = nalgebra_glm::dot(&v0, &v1);
    let d11 = nalgebra_glm::dot(&v1, &v1);
    let d20 = nalgebra_glm::dot(&v2, &v0);
    let d21 = nalgebra_glm::dot(&v2, &v1);
    let denom = d00 * d11 - d01 * d01;
    if denom.abs() < f32::EPSILON {
        return (-1.0, -1.0, -1.0);
    }
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    (u, v, w)
}

pub fn barycentric_test(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3) -> bool {
    let (u, v, w) = get_barycentric_coords(p, a, b, c);
    u >= -0.6 && v >= -0.6 && w >= -0.6
}
