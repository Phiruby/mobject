use crate::shapes::UBO;
use glm;
use std::sync::LazyLock;
use std::time::Instant;
static START_TIME: LazyLock<Instant> = LazyLock::new(|| Instant::now());

pub fn rotate(ubo: &mut UBO, degreed_per_minute: u64) {
    let current_time = Instant::now();
    let delta_seconds = current_time.elapsed().as_secs() - START_TIME.elapsed().as_secs();
    ubo.model = glm::ext::rotate(
        &ubo.model,
        (degreed_per_minute * delta_seconds / 60) as f32,
        glm::Vector3::new(0.0, 0.0, 1.0),
    );
}
