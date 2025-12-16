use mobject::Scene;
use mobject::shapes::{Shape, Triangle};
fn main() {
    println!("Hello, world!");
    let triangle = Box::new(Triangle::default());
    println!("{:?}", triangle.vertices2d());
    let mobjects: Vec<Box<dyn Shape>> = vec![triangle];
    let scene = Scene::new(Some(mobjects));
    scene.main_loop();
}
