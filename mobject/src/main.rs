use mobject::Scene;
// use mobject::shapes::surface::Surface3D;
use mobject::shapes::{Cube, objects::Cloth};
use nalgebra_glm::Vec3;


fn main() {
    let rect = Cube::default()
        .finish_construction()
        .make_rigid()
        .with_velocities(vec![Vec3::new(0.0, 1.5, 0.0); 8]);
    // dbg!(rect.entity.get_vertices().iter().map(|v| v.position).collect::<Vec<Vec3>>());
    let c = Cloth::example();
    // dbg!(c.entity.get_vertices().iter().map(|v| v.position).collect::<Vec<Vec3>>());
    let mut scene = Scene::new();
    scene.add_baseplate();
    scene.wait(3);
    // scene.wait(3);
    // scene.wait(10);
    // scene.wait(5);
    let r = scene.add(rect);
    let c = scene.add(c);
    // scene.attach(c, 30, r, 0);
    // scene.attach(c, 36, r, 1);
    // let cloth_indices: Vec<usize> = (30..=36).collect();
    // let cube_indices = Cube::edge_indices_top_front(5);

    // for (ci, ri) in cloth_indices.iter().zip(cube_indices.iter()) {
    //     scene.attach(c, *ci, r, *ri);
    // }
    // scene.wait(8);
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
    //     .move_to(glm::make_vec3(&[4.0, 1.0, 0.75]))
    //     .with_duration(2.0);

    // scene.motion(camera_motion);
    scene.main_loop();
}
