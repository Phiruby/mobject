use crate::shapes::UBO;
use glm;
use std::sync::LazyLock;
use std::time::Instant;
static START_TIME: LazyLock<Instant> = LazyLock::new(|| Instant::now());

pub fn rotate(ubo: &mut UBO, degreed_per_minute: f32) {
    let delta_seconds = START_TIME.elapsed().as_secs_f32();
    let angle = glm::builtin::radians(degreed_per_minute * delta_seconds / 60.0);
    let identity = glm::mat4(
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
    ubo.model = glm::ext::rotate(&identity, angle, glm::Vector3::new(0.0, 0.0, 1.0));
}
