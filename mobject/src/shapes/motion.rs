// use std::ops::Deref;

// use crate::shapes::{ShapeMotion, BuiltShape, Shape, UBO};
// use ash::{Device, vk::PhysicalDeviceMemoryProperties};
// use nalgebra_glm as glm;
// use crate::motion_shape_construction;
// pub struct Rotate {
//     angular_velocity: f32,
//     axis: glm::Vec3,
//     mobject: Box<dyn Shape>,
// }

// pub struct BuiltRotate {
//     angular_velocity: f32,
//     mobject: Box<dyn BuiltShape>,
//     ubo: UBO,
//     axis: glm::Vec3,
//     angle: f32
// }

// motion_shape_construction!(BuiltRotate);

// impl Deref for Rotate {
//     type Target = Box<dyn Shape>;
//     fn deref(&self) -> &Self::Target {
//         &self.mobject
//     }
// }

// impl Deref for BuiltRotate {
//     type Target = Box<dyn BuiltShape>;
//     fn deref(&self) -> &Self::Target {
//         &self.mobject
//     }
// }

// impl Rotate {
//     pub fn new(angular_velocity: f32, axis: glm::Vec3, mobject: Box<dyn Shape>) -> Box<dyn Shape> {
//         Box::new(Self {
//             angular_velocity,
//             axis,
//             mobject
//         })
//     }
// }

// impl Shape for Rotate {
//     fn build(self: Box<Self>, device: &Device, mem_properties: PhysicalDeviceMemoryProperties) -> Box<dyn BuiltShape> {
//         let build = self.mobject.build(device, mem_properties);
//         let ubo = build.get_ubo_contents().clone();
//         Box::new(BuiltRotate { mobject: build, angular_velocity: self.angular_velocity, axis: self.axis, angle: 0.0, ubo })
//     }

//     fn texture_path(&self) -> Option<&str> {
//         self.mobject.texture_path()
//     }
// }

// impl ShapeMotion for BuiltRotate {
//     fn update(&mut self) {
//         let new_model = glm::rotate(&self.ubo.model, self.angular_velocity, &self.axis);
//         self.ubo.model = new_model;
//     }
//     fn get_ubo_contents(&self) -> &UBO {
//         &self.ubo
//     }
// }
