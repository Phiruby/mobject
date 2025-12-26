use mobject::Scene;
use mobject::shapes::{Rectangle, Shape, Triangle};
fn main() {
    println!("Hello, world!");
    let triangle = Box::new(Triangle::default());
    let shape = Box::new(Rectangle::default());
    println!("{:?}", shape.vertices2d());
    let mobjects: Vec<Box<dyn Shape>> = vec![shape];
    let mut scene = Scene::new(Some(mobjects));
    scene.main_loop();
}
