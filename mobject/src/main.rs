use mobject::Scene;
use mobject::shapes::{ObjModel, Points, Rectangle, Shape, Triangle, Vertex2D};
fn main() {
    println!("Hello, world!");
    // let triangle = Box::new(Triangle::default());
    let shape = Box::new(Rectangle::default().include_texture("textures/basic.jpg"));
    let texture_less = Box::new(Triangle::default());
    // let obj = Box::new(ObjModel::new("models/room.obj"));
    // let mobjects: Vec<Box<dyn Shape>> = vec![shape];
    // let bezier_points = Box::new(Points::default());
    // let mobjects: Vec<Box<dyn Shape>> = vec![bezier_points];
    let mut scene = Scene::new();
    scene.add(texture_less);
    scene.wait(5);
    scene.add(shape);
    scene.wait(3);
    scene.main_loop();
    // scene.add(bezier_points);
    // scene.add(bezier_points);
    // scene.wait(10);
    // scene.main_loop();
}
