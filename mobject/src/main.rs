use mobject::Scene;
use mobject::shapes::{ObjModel, Points, Rectangle, Shape, Triangle, Vertex2D};
fn main() {
    println!("Hello, world!");
    // let triangle = Box::new(Triangle::default());
    let shape = Box::new(Rectangle::default().include_texture("textures/basic.jpg"));
    let texture_less = Box::new(Triangle::default());
    let obj = Box::new(ObjModel::load("models/room.obj").include_texture("textures/viking_room.png"));
    let mut scene = Scene::new();
    scene.add(texture_less);
    scene.wait(5);
    scene.add(shape);
    scene.wait(3);
    scene.add(obj);
    scene.wait(3);
    scene.main_loop();
}
