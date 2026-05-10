# Mobject
Create physics animations that break your intuition.

## Demos
Drop a piece of cloth on a cube:
```rs
use mobject::Scene;
use mobject::shapes::{Cube, objects::Cloth};
use nalgebra_glm::Vec3;

fn main() {
    let rect = Cube::default()
        .finish_construction()
        .make_rigid();
    let c = Cloth::example();
    let mut scene = Scene::new();
    scene.add_baseplate();
    let _r = scene.add(rect);
    scene.wait(3);
    let _c = scene.add(c);

    scene.wait(10);
    let camera_motion = scene
        .camera()
        .move_to(Vec3::new(2.0, 1.0, 2.75))
        .with_duration(2.0);

    scene.motion(camera_motion);
    scene.main_loop();
}
```
Result:

https://github.com/user-attachments/assets/391e8849-33fc-4386-9a86-1849130c19ae

Drop a cube on a piece of cloth anchored around its four corners:
```rs
use mobject::Scene;
use mobject::shapes::{Cube, objects::Cloth};
use nalgebra_glm::Vec3;

fn main() {
    let rect = Cube::default()
        .finish_construction()
        .make_rigid();
    let c = Cloth::example()
        .with_inverse_masses(vec![5.0; 100])
        .anchor(vec![0, 9, 90, 99]);
    let mut scene = Scene::new();
    let _c = scene.add(c);
    scene.wait(2);
    let _r = scene.add(rect);

    scene.wait(3);
    let camera_motion = scene
        .camera()
        .move_to(Vec3::new(2.0, 1.0, 2.75))
        .with_duration(2.0);

    scene.motion(camera_motion);
    scene.main_loop();
}
```
Result:

https://github.com/user-attachments/assets/4df8bb3d-ad1f-4ba1-aecb-0b462ad1aa4b

## Methods

This runs on **position-based dynamics**: instead of manually handling forces, we compute dynamics of objects based on their current position and how it changed from previous frames while including constraints (see link below for full details).

The following reads were used in implementing this engine:

1. Position based dynamics: https://matthias-research.github.io/pages/publications/posBasedDyn.pdf
2. Shape Matching (to mantain rigid bodies): https://matthias-research.github.io/pages/publications/Physically_Based_Shape_Matching___SCA_2022.pdf
3. A general overview of force-based physics simulators (the main takeaway used here is the section on collisions- these were helpful to distinguish between 2D and 3D objects colliding for example): https://www.cs.cmu.edu/~baraff/pbm/rigid1.pdf

## Things to address in the future
1. Shadows display the "peter-panning" effect (see https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping)
2. Aliasing artifacts: can improve by adding filtering / mipmaps
3. `dt` is hardcoded in physics sim: each frame has a fixed `dt` increment
