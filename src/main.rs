use mobject::Scene;
use mobject::shapes::{ObjModel, Rectangle, Shape, Triangle};
fn main() {
    println!("Hello, world!");
    let triangle = Box::new(Triangle::default());
    let shape = Box::new(Rectangle::default());
    let obj = Box::new(ObjModel::new("models/room.obj"));
    let mobjects: Vec<Box<dyn Shape>> = vec![shape, triangle, obj];
    let mut scene = Scene::new(Some(mobjects));
    scene.main_loop();
}
