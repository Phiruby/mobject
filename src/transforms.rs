use crate::shapes::UBO;
use nalgebra_glm as glm;
use std::sync::LazyLock;
use std::time::Instant;
static START_TIME: LazyLock<Instant> = LazyLock::new(|| Instant::now());

pub fn rotate(ubo: &mut UBO, degreed_per_minute: f32) {
    let delta_seconds = START_TIME.elapsed().as_secs_f32();
    let degrees = glm::vec1(degreed_per_minute * delta_seconds / 60.0);
    let angle = glm::radians(&degrees).x;
    let identity = glm::mat4(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
    let axis = glm::vec3(0.0, 0.0, 1.0);
    ubo.model = glm::rotate(&identity, angle, &axis);
}
