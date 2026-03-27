use mobject::Scene;
use mobject::shapes::{ObjModel, Points, Rectangle, Rotate, Shape, Triangle, Vertex2D};
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
    let rotating_obj = Rotate::new(
        0.001,
        glm::make_vec3(&[0.0, 0.0, 1.0]), obj);

    let rotating_pts = Rotate::new(
        0.005,
        glm::make_vec3(&[0.0, 0.0, 1.0]),
        points
    );

    scene.add(rotating_obj);
    scene.wait(10);
    scene.add(rotating_pts);
    scene.wait(3);
    scene.main_loop();
}
