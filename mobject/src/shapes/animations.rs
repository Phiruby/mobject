use crate::{scene::{MobjectId, Scene}, shapes::BuiltShape};
use nalgebra_glm::Vec3;
pub trait Animation {
    fn update(&mut self, dt: f32, obj: &mut Box<dyn BuiltShape>);
    fn is_finished(&self) -> bool;
    fn get_target(&self) -> MobjectId;
}

pub struct AnimationProxy {
    pub id: MobjectId
}

struct Rotate {
    target: MobjectId,
    speed: f32,
    axis: Vec3,
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

impl AnimationProxy {

    pub fn rotate(self, axis: Vec3, speed: f32) -> Box<dyn Animation> {
        Box::new(Rotate {
            target: self.id,
            axis,
            speed,
        })
    }
}
