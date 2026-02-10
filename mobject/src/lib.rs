pub mod buffers;
pub mod c_utils;
pub mod device;
pub mod render_pass;
pub mod shaders;
pub mod shapes;
pub mod swapchain;
pub mod texture;
pub mod transforms;
pub mod window;
pub mod scene;
pub mod pipelines;
pub use scene::Scene;
// TODO: set to num swapchain images instead of hardcoding to my machine
const MAX_FRAMES_IN_FLIGHT: u32 = 3;
