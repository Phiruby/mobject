use ash::Instance;
use glfw::{self, Glfw, Window, ffi::VkSurfaceKHR};

pub fn create_glfw_window(width: u32, height: u32) -> Window {
    let mut wrapper = glfw::init_no_callbacks().unwrap();
    wrapper.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));
    wrapper.window_hint(glfw::WindowHint::Resizable(glfw::ffi::GLFW_TRUE));
    let (window, receiver) =
        wrapper.create_window(width, height, "Mobject", glfw::WindowMode::Windowed);
    window
}

pub fn create_surface(instance: &Instance, window: &Window) -> VkSurfaceKHR {
    let glfw_vk_instance = glfw::ffi::VkInstance::from(instance.handle());
    let mut surface: *mut VkSurfaceKHR = VkSurfaceKHR::default();
    unsafe { window.create_window_surface(glfw_vk_instance, std::ptr::null(), &mut surface) };
    unsafe { *surface }
}
