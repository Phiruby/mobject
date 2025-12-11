use ash::{Instance, vk::Handle};
use glfw::{self, Glfw, PWindow, ffi::VkSurfaceKHR};

pub fn create_glfw_window(width: u32, height: u32) -> (PWindow, Vec<String>) {
    let mut wrapper = glfw::init_no_callbacks().unwrap();
    wrapper.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    wrapper.window_hint(glfw::WindowHint::Resizable(true));
    let possible_window =
        wrapper.create_window(width, height, "Mobject", glfw::WindowMode::Windowed);
    let (window, receiver) = possible_window.unwrap();
    let required_extensions = match wrapper.get_required_instance_extensions() {
        Some(x) => x,
        _ => Vec::new(),
    };
    (window, required_extensions)
}

pub fn create_surface(instance: &Instance, window: &PWindow) -> VkSurfaceKHR {
    let glfw_vk_instance = instance.handle().as_raw() as glfw::ffi::VkInstance;
    let mut surface = VkSurfaceKHR::default();
    let mut surface_ptr: *mut VkSurfaceKHR = &mut surface;
    unsafe { window.create_window_surface(glfw_vk_instance, std::ptr::null(), surface_ptr) };
    surface
}
