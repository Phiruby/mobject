use mobject::Scene;
use mobject::shapes::{Cube, objects::Cloth};
use nalgebra_glm::Vec3;

fn main() {
    let rect = Cube::default()
        .finish_construction()
        .make_rigid();
    let c = Cloth::example()
        .with_inverse_masses(vec![5.0; 100])
        .anchor(vec![0, 9, 90, 99]);
    let mut scene = Scene::new();
    scene.wait(15);
    let _c = scene.add(c);
    scene.wait(2);
    let _r = scene.add(rect);

    scene.wait(3);
    let camera_motion = scene
        .camera()
        .move_to(Vec3::new(2.0, 1.0, 2.75))
        .with_duration(2.0);

    scene.motion(camera_motion);
    scene.main_loop();
}
