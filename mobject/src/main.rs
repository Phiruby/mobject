use mobject::Scene;
use mobject::shapes::{ObjModel, Points, Rectangle, Shape, Triangle, Vertex2D};
use nalgebra_glm as glm;
fn main() {
    println!("Hello, world!");
    // let triangle = Box::new(Triangle::default());
    let shape = Box::new(Rectangle::default().include_texture("textures/basic.jpg"));
    let texture_less = Box::new(Triangle::default());
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
    scene.main_loop();
}
