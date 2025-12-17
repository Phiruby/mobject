use std::ffi::c_void;
use std::ptr::copy_nonoverlapping;

use crate::shapes::{self, Shape, UBO, Vertex2D};
use crate::{MAX_FRAMES_IN_FLIGHT, buffers, render_pass};
use ash::vk::{
    Buffer, CommandBuffer, CommandBufferResetFlags, DeviceMemory, Extent2D, Framebuffer,
    MemoryMapFlags, Pipeline, PresentInfoKHR, Queue, RenderPass, SubmitInfo, SwapchainKHR,
};
use ash::{Device, khr::swapchain};
use ash::{
    Instance,
    vk::{self, Fence, FenceCreateInfo, Handle, Semaphore, SemaphoreCreateInfo, StructureType},
};
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

pub struct Sync {
    pub image_available: Semaphore,
    pub render_finished: Semaphore,
    pub in_flight: Fence,
}

pub fn create_sync_objects(device: &Device) -> Vec<Sync> {
    let semaphore_info = SemaphoreCreateInfo {
        s_type: StructureType::SEMAPHORE_CREATE_INFO,
        ..Default::default()
    };
    let fence_info = FenceCreateInfo {
        s_type: StructureType::FENCE_CREATE_INFO,
        flags: vk::FenceCreateFlags::SIGNALED, // start signaled (to draw first frame)
        ..Default::default()
    };
    (0..MAX_FRAMES_IN_FLIGHT)
        .map(|_| Sync {
            image_available: unsafe { device.create_semaphore(&semaphore_info, None) }.unwrap(),
            render_finished: unsafe { device.create_semaphore(&semaphore_info, None) }.unwrap(),
            in_flight: unsafe { device.create_fence(&fence_info, None) }.unwrap(),
        })
        .collect()
}

pub fn fill_vertex_buffer(device: &Device, memory: DeviceMemory, vertices: &[Vertex2D]) {
    let memory_loc =
        unsafe { device.map_memory(memory, 0, vk::WHOLE_SIZE, MemoryMapFlags::empty()) }.unwrap();
    unsafe {
        let dst = memory_loc as *mut Vertex2D;
        copy_nonoverlapping(vertices.as_ptr(), dst, vertices.len());
        device.unmap_memory(memory);
    }
}

pub fn fill_index_buffer(device: &Device, memory: DeviceMemory, indices: &[u32]) {
    let memory_loc =
        unsafe { device.map_memory(memory, 0, vk::WHOLE_SIZE, MemoryMapFlags::empty()) }.unwrap();
    unsafe {
        let dst = memory_loc as *mut u32;
        copy_nonoverlapping(indices.as_ptr(), dst, indices.len());
        device.unmap_memory(memory);
    }
}

pub fn fill_uniform_buffer(device: &Device, memory_loc: *mut c_void, ubo: &UBO) {
    unsafe {
        let dst = memory_loc as *mut UBO;
        copy_nonoverlapping(ubo as *const UBO, dst, 1);
    }
}
