use std::ptr::copy_nonoverlapping;

use crate::shapes::{Shape, Vertex2D};
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

fn mobjects_to_vertices(mobjects: &[Box<dyn Shape>]) -> Vec<Vertex2D> {
    let mut vertices: Vec<Vertex2D> = Vec::new();
    for i in (0..mobjects.len()) {
        let shape_vertices = mobjects[i].vertices2d();
        shape_vertices.iter().for_each(|f| vertices.push(f.clone()));
    }
    vertices
}

fn fill_vertex_buffer(device: &Device, memory: DeviceMemory, vertices: &[Vertex2D]) {
    let memory_loc =
        unsafe { device.map_memory(memory, 0, vk::WHOLE_SIZE, MemoryMapFlags::empty()) }.unwrap();
    unsafe {
        let dst = memory_loc as *mut Vertex2D;
        copy_nonoverlapping(vertices.as_ptr(), dst, vertices.len());
        device.unmap_memory(memory);
    }
}

pub fn draw_frame(
    vertex_buffer: Buffer,
    vertex_buffer_memory: DeviceMemory,
    mobjects: &[Box<dyn Shape>],
    sync: &Sync,
    device: &Device,
    swapchain_device: &swapchain::Device,
    swapchain: SwapchainKHR,
    command_buffer: CommandBuffer,
    render_pass: RenderPass,
    framebuffers: &[Framebuffer],
    extent: Extent2D,
    graphics_pipeline: Pipeline,
    graphics_queue: Queue,
    present_queue: Queue,
) {
    unsafe { device.wait_for_fences(&[sync.in_flight], true, u64::MAX) }.unwrap();
    unsafe { device.reset_fences(&[sync.in_flight]) }.unwrap();
    let vertices = mobjects_to_vertices(mobjects);
    fill_vertex_buffer(device, vertex_buffer_memory, &vertices);
    let (image_index, _suboptimal) = unsafe {
        swapchain_device.acquire_next_image(
            swapchain,
            u64::MAX,
            sync.image_available,
            Fence::null(),
        )
    }
    .unwrap();
    unsafe { device.reset_command_buffer(command_buffer, CommandBufferResetFlags::empty()) }
        .unwrap();

    buffers::record_command_buffer(
        device,
        command_buffer,
        vertex_buffer,
        vertices.len() as u32,
        image_index,
        render_pass,
        framebuffers,
        extent,
        graphics_pipeline,
    );
    let semaphores = vec![sync.image_available];
    let wait_stages = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
    let signal_semaphores = vec![sync.render_finished];
    let submit_info = SubmitInfo {
        s_type: StructureType::SUBMIT_INFO,
        wait_semaphore_count: 1,
        p_wait_semaphores: semaphores.as_ptr(),
        p_wait_dst_stage_mask: wait_stages.as_ptr(),
        command_buffer_count: 1,
        p_command_buffers: &command_buffer,
        signal_semaphore_count: 1,
        p_signal_semaphores: signal_semaphores.as_ptr(),
        ..Default::default()
    };
    unsafe { device.queue_submit(graphics_queue, &[submit_info], sync.in_flight) }.unwrap();
    let swapchains = vec![swapchain];
    let present_info = PresentInfoKHR {
        s_type: StructureType::PRESENT_INFO_KHR,
        wait_semaphore_count: 1,
        p_wait_semaphores: signal_semaphores.as_ptr(),
        swapchain_count: 1,
        p_swapchains: swapchains.as_ptr(),
        p_image_indices: &image_index,
        ..Default::default()
    };
    unsafe { swapchain_device.queue_present(present_queue, &present_info) }.unwrap();
}
