use nalgebra_glm::{Mat3x3, Vec3, Quat};
struct OdeSolver ();

struct StateDerivative {
    velocity: Vec3,
    rotation_prime: Mat3x3,
    force: Vec3,
    torque: Vec3
}

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

impl std::ops::Mul<f32> for StateDerivative {
    type Output = StateDerivative;
    fn mul(self, rhs: f32) -> Self::Output {
        StateDerivative { velocity: self.velocity * rhs, rotation_prime: self.rotation_prime * rhs, force: self.force * rhs, torque: self.torque * rhs }
    }
}

impl std::ops::Add<StateDerivative> for PhysicalState {
    type Output = PhysicalState;
    fn add(self, rhs: StateDerivative) -> Self::Output {
        PhysicalState {
            mass: self.mass,
            intertia_body: self.intertia_body,
            intertia_body_inv: self.intertia_body_inv,
            orientation: self.orientation,
            position: self.position,
            momentum: self.momentum,
            angular_momentum: self.angular_momentum,
            inertia_inv: self.inertia_inv,
            velocity: self.velocity + rhs.velocity,
            angular_velocity: self.angular_velocity + rhs.rotation_prime * self.orientation,
            force: self.force + rhs.force,
            torque: self.torque + rhs.torque
        }
    }
}

impl PhysicalState {
    fn differentiate(&self) -> StateDerivative {
        StateDerivative { velocity: self.velocity, rotation_prime: Mat3x3::identity(), force: nalgebra_glm::make_vec3(&[0.0, 0.0, 0.0]), torque: nalgebra_glm::make_vec3(&[0.0, 0.0, 0.0]) }
    }
}

impl OdeSolver {
    pub fn solve(&self, state: PhysicalState) -> PhysicalState {
        let dydt = state.differentiate();
        state + dydt * 0.01
    }
}
