use mobject::Scene;
use mobject::shapes::{Rectangle, Shape, Triangle};
fn main() {
    println!("Hello, world!");
    let triangle = Box::new(Triangle::default());
    let shape = Box::new(Rectangle::default());
    println!("{:?}", triangle.vertices2d());
    let mobjects: Vec<Box<dyn Shape>> = vec![triangle, shape];
    let scene = Scene::new(Some(mobjects));
    scene.main_loop();
}
