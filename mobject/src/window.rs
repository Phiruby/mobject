use std::ffi::c_void;
use std::ptr::copy_nonoverlapping;

use crate::shapes::RenderVertex;
use crate::MAX_FRAMES_IN_FLIGHT;
use ash::vk::{
    DeviceMemory, ImageMemoryBarrier, MemoryMapFlags
};
use ash::Device;
use ash::{
    Instance,
    vk::{self, Fence, FenceCreateInfo, Handle, Semaphore, SemaphoreCreateInfo, StructureType},
};
use glfw::{GlfwReceiver, WindowEvent};
use glfw::{self, PWindow, ffi::VkSurfaceKHR};
pub fn create_glfw_window(width: u32, height: u32) -> (PWindow, Vec<String>, GlfwReceiver<(f64, WindowEvent)>) {
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
    (window, required_extensions, receiver)
}

pub fn create_surface(instance: &Instance, window: &PWindow) -> VkSurfaceKHR {
    let glfw_vk_instance = instance.handle().as_raw() as glfw::ffi::VkInstance;
    let mut surface = VkSurfaceKHR::default();
    let surface_ptr: *mut VkSurfaceKHR = &mut surface;
    unsafe { window.create_window_surface(glfw_vk_instance, std::ptr::null(), surface_ptr) };
    surface
}

pub struct Sync {
    pub image_available: Semaphore,
    pub render_finished: Semaphore,
    pub in_flight: Fence,
}

pub fn create_image_barriers<'a>(
    shadow_images: &[vk::Image],
) -> Vec<ImageMemoryBarrier<'a>> {
    let mut barriers: Vec<ImageMemoryBarrier> = Vec::new();
    for i in 0..shadow_images.len() {
        let shadow_barrier = vk::ImageMemoryBarrier {
        old_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        new_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        src_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        dst_access_mask: vk::AccessFlags::SHADER_READ,
        image: shadow_images[i],
        subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        barriers.push(shadow_barrier);
    }
    barriers
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

pub fn fill_vertex_buffer(device: &Device, memory: DeviceMemory, vertices: &[RenderVertex]) {
    let memory_loc =
        unsafe { device.map_memory(memory, 0, vk::WHOLE_SIZE, MemoryMapFlags::empty()) }.unwrap();
    unsafe {
        let dst = memory_loc as *mut RenderVertex;
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

pub fn fill_uniform_buffer<T>(memory_loc: *mut c_void, ubo: &T) {
    unsafe {
        let dst = memory_loc as *mut T;
        copy_nonoverlapping(ubo as *const T, dst, 1);
    }
}
