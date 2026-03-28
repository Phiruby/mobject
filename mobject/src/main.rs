use mobject::Scene;
use mobject::shapes::{ObjModel, Points};
use nalgebra_glm as glm;
fn main() {
    let obj = Box::new(ObjModel::load("models/room.obj").include_texture("textures/viking_room.png"));
    let mut scene = Scene::new();

    let points = Box::new(
        Points::default()
        .include_texture("textures/basic.jpg")
    );
    scene.wait(3);
    let pts = scene.add(points);
    let anim = scene.animate(pts).rotate(glm::make_vec3(&[0.0, 0.0, 1.0]), 0.005);
    scene.play(anim);

    scene.wait(5);
    let obj = scene.add(obj);
    scene.wait(5);
    let anim = scene.animate(obj).rotate(
        glm::make_vec3(&[0.0, 1.0, 0.0]), 0.001
    );
    scene.play(anim);
    scene.wait(3);
    let camera_motion = scene
        .camera()
        .move_to(glm::make_vec3(&[4.0, 1.0, 5.0]))
        .with_duration(2.0);

    scene.motion(camera_motion);
    scene.main_loop();
}
