use mobject::Scene;
// use mobject::shapes::surface::Surface3D;
use mobject::shapes::{ObjModel, Points, Rectangle, Cube, objects::Cloth};
use nalgebra_glm as glm;
use nalgebra_glm::{Mat4, Vec3};
use mobject::physics::constraints::{PhysicsConstraint, StaticConstraint, StretchConstraint};


fn main() {
    let rect = Box::new(
        Cube::default()
        .with_velocities(vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ])
        // .make_static()
        .anchor(vec![0, 1])
        .rigid_body()
    );
    let c = Box::new(Cloth::default().anchor(vec![0, 9, 90, 99]));
    let mut scene = Scene::new();
    // scene.wait(3);
    // scene.add(c);
    scene.wait(3);
    scene.add(rect);
    // let obj = Box::new(ObjModel::load("models/room.obj").include_texture("textures/viking_room.png"));

    // let rect = Box::new(
    //     Rectangle::default()
    // );

    // let points = Box::new(
    //     Points::default()
    //     .include_texture("textures/basic.jpg")
    // );
    // scene.wait(3);
    // let pts = scene.add(points);
    // let anim = scene.animate(pts).rotate(glm::make_vec3(&[0.0, 0.0, 1.0]), 0.005);
    // scene.play(anim);
    // scene.wait(3);
    // let r = scene.add(rect);
    // let anim = scene.animate(r).rotate(
    //     glm::make_vec3(&[0.0, 0.0, 1.0]), 0.001
    // );
    // scene.play(anim);
    // scene.wait(5);
    // let obj = scene.add(obj);
    // scene.wait(5);
    // let anim = scene.animate(obj).rotate(
    //     glm::make_vec3(&[0.0, 1.0, 0.0]), 0.001
    // );
    // scene.play(anim);
    // scene.wait(3);
    // let camera_motion = scene
    //     .camera()
    //     .move_to(glm::make_vec3(&[4.0, 1.0, 5.0]))
    //     .with_duration(2.0);

    // scene.motion(camera_motion);
    scene.main_loop();
}
