use std::time::Instant;

use crate::{scene::{MobjectId, Scene}, shapes::{BuiltShape, GlobalUBO}};
use nalgebra_glm::{Vec3, Mat4};
pub trait Animation {
    fn update(&mut self, dt: f32, obj: &mut Box<dyn BuiltShape>);
    fn is_finished(&self) -> bool;
    fn get_target(&self) -> MobjectId;
}

pub struct AnimationProxy {
    pub id: MobjectId
}

pub struct CameraProxy ();


struct Rotate {
    target: MobjectId,
    speed: f32,
    axis: Vec3,
}

pub struct CameraMotion {
    target_position: Vec3,
    duration: f32,
    complete: bool
}

pub struct CameraAnimation {
    pub anim: CameraMotion,
    pub start_time: Instant,
    pub init_position: Vec3
}

impl Animation for Rotate {
    fn update(&mut self, dt: f32, obj: &mut Box<dyn BuiltShape>) {
        obj.rotate(&self.axis, self.speed);
    }

    fn is_finished(&self) -> bool {
        // TODO: this is continuous rotation
        false
    }
    fn get_target(&self) -> MobjectId {
        self.target
    }
}

impl CameraMotion {
    pub fn with_duration(self, duration: f32) -> Self {
        Self {
            duration,
            ..self
        }
    }
    pub fn update(&mut self, ubo: &mut GlobalUBO, init_position: Vec3, dt: f32) {
        // TODO: time may not be exactly 1; may want to clip

        let new_camera_position = init_position * (1.0 - dt / self.duration) + (dt / self.duration) * self.target_position;
        let new_view = nalgebra_glm::look_at(
            &new_camera_position,
            &nalgebra_glm::make_vec3(&[0.0, 0.0, 0.0]),
            &nalgebra_glm::make_vec3(&[0.0, 0.0, 1.0])
        );
        ubo.camera_position = new_camera_position;
        ubo.view = new_view;
        self.complete = (dt / self.duration) >= 1.0;
    }

    pub fn is_finished(&self) -> bool {
        self.complete
    }
}

impl CameraProxy {
    pub fn move_to(&self, target: Vec3) -> CameraMotion {
        CameraMotion { target_position: target, duration: 1.0, complete: false }
    }
}

impl AnimationProxy {

    pub fn rotate(self, axis: Vec3, speed: f32) -> Box<dyn Animation> {
        Box::new(Rotate {
            target: self.id,
            axis,
            speed,
        })
    }
}
