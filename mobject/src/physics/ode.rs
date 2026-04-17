use nalgebra_glm::{Mat3x3, Vec3, Quat};
struct OdeSolver ();

/// Represents the state of a physical object
/// Most metrics are self-explanatory. But important clarifications:
/// 1. inertia_body and inertia_body_inv are the inertia tensor of the object about its center of mass
/// (aka its intertia in "body space")
struct PhysicalState {
    mass: f32,
    intertia_body: Mat3x3,
    intertia_body_inv: Mat3x3,
    orientation: Quat,
    position: Vec3,
    momentum: Vec3,
    angular_momentum: Vec3,
    // derived
    inertia_inv: Mat3x3,
    velocity: Vec3,
    angular_velocity: Vec3,
    // computed
    force: Vec3,
    torque: Vec3
}

trait Differentiate {
    fn differentiate(&self) -> Self;
}

impl Differentiate for PhysicalState {
    fn differentiate(&self) -> Self {
        todo!()
    }
}

impl OdeSolver {
    pub fn solve(&self) {

    }
}
