use mobject::Scene;
use mobject::shapes::{ObjModel, Points, Rectangle, Shape, Triangle, Vertex2D};
fn main() {
    println!("Hello, world!");
    // let triangle = Box::new(Triangle::default());
    let shape = Box::new(Rectangle::default());
    // let obj = Box::new(ObjModel::new("models/room.obj"));
    let mobjects: Vec<Box<dyn Shape>> = vec![shape];
    let bezier_points = Box::new(Points::default());
    // let mobjects: Vec<Box<dyn Shape>> = vec![bezier_points];
    let mut scene = Scene::new(Some(mobjects));
    // scene.add(bezier_points);
    scene.wait(10);
    scene.add(bezier_points);
    scene.main_loop();
}
