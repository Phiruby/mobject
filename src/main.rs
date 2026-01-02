use mobject::Scene;
use mobject::shapes::{Rectangle, Shape, Triangle};
fn main() {
    println!("Hello, world!");
    let triangle = Box::new(Triangle::default());
    let shape = Box::new(Rectangle::default());
    let mobjects: Vec<Box<dyn Shape>> = vec![shape, triangle];
    let mut scene = Scene::new(Some(mobjects));
    scene.main_loop();
}
