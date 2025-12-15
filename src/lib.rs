pub mod buffers;
pub mod c_utils;
pub mod device;
pub mod render_pass;
pub mod shaders;
pub mod shapes;
pub mod swapchain;
pub mod window;

use ash::vk::{
    self, ApplicationInfo, Buffer, CommandBuffer, DeviceMemory, Extent2D, Framebuffer, Handle,
    InstanceCreateInfo, Pipeline, RenderPass, StructureType, SurfaceKHR, SwapchainKHR,
};
use ash::{Device, Entry, Instance, khr, khr::surface};
use c_utils::Utf8Pointer;
use device::QueueFamilies;
use glfw::PWindow;
use shapes::Shape;
use std::char::MAX;
use std::ffi::CString;
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
const DEVICE_EXTENSIONS: [&str; 1] = ["VK_KHR_swapchain"];
// TODO: set to num swapchain images instead of hardcoding to my machine
const MAX_FRAMES_IN_FLIGHT: u32 = 3;
pub struct Scene {
    sync: Vec<window::Sync>,
    device: Device,
    swapchain_device: khr::swapchain::Device,
    window: PWindow,
    swapchain: SwapchainKHR,
    command_buffer: Vec<CommandBuffer>,
    render_pass: RenderPass,
    framebuffers: Vec<Framebuffer>,
    vertex_buffers: Vec<Buffer>,
    vertex_buffer_memory: Vec<DeviceMemory>,
    extent: Extent2D,
    graphics_pipeline: Pipeline,
    queue_families: QueueFamilies,
    mobjects: Vec<Box<dyn Shape>>,
}

impl Scene {
    pub fn new(mobjects: Option<Vec<Box<dyn Shape>>>) -> Self {
        let entry = unsafe { Entry::load().unwrap() };
        let (window, required_instance_extensions) = window::create_glfw_window(700, 700);
        let instance = create_vk_instance(&entry, Some(required_instance_extensions));
        let surface_instance = surface::Instance::new(&entry, &instance);
        let surface = window::create_surface(&instance, &window);
        let ash_surface = SurfaceKHR::from_raw(surface as u64);
        let physical_device = device::select_physical_device(&instance);
        let queue_families =
            QueueFamilies::new(&instance, &surface_instance, ash_surface, physical_device);
        let mut swapchain_capabilities =
            swapchain::query_support(physical_device, &surface_instance, ash_surface);
        let surface_format = swapchain_capabilities.choose_surface_format();
        let present_mode = swapchain_capabilities.choose_present_mode();
        let extent = swapchain_capabilities.choose_extent(&window);
        let logical_device = device::create_logical_device(
            &queue_families,
            &instance,
            physical_device,
            None,
            Some(DEVICE_EXTENSIONS.to_vec()),
        );
        let swapchain_device = khr::swapchain::Device::new(&instance, &logical_device);
        let swapchain = swapchain::create(
            &swapchain_device,
            present_mode,
            &swapchain_capabilities.capabilities,
            ash_surface,
            surface_format,
            &queue_families,
            extent,
        );
        let swapchain_images = swapchain::acquire_images(swapchain, &swapchain_device);
        println!("{:?} <-- total swapchain images", swapchain_images.len());
        let swapchain_imageviews =
            swapchain::create_image_views(&logical_device, &swapchain_images, surface_format);

        let render_pass = render_pass::create(surface_format, &logical_device);
        let graphics_pipeline =
            shaders::create_graphics_pipeline(&logical_device, extent, render_pass);
        let framebuffers = buffers::create_frame_buffers(
            &logical_device,
            render_pass,
            &swapchain_imageviews,
            extent,
        );
        let pool = buffers::create_command_pool(&logical_device, &queue_families);
        let command_buffer = buffers::create_command_buffers(pool, &logical_device);
        let sync = window::create_sync_objects(&logical_device);
        // TODO: unhardcode the max 10 vertices
        let vertex_buffers = buffers::create_vertex_buffers(&logical_device, 10);
        let physical_device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let vertex_buffer_memories = buffers::allocate_vertex_buffers_memory(
            &vertex_buffers,
            &logical_device,
            physical_device_memory_properties,
        );
        Self {
            sync,
            device: logical_device,
            swapchain_device,
            window,
            command_buffer,
            swapchain,
            render_pass,
            framebuffers,
            vertex_buffers,
            vertex_buffer_memory: vertex_buffer_memories,
            extent,
            graphics_pipeline,
            queue_families,
            mobjects: mobjects.unwrap_or_default(),
        }
    }

    pub fn main_loop(&self) {
        let graphics_queue = unsafe {
            self.device
                .get_device_queue(0, self.queue_families.graphics_index as u32)
        };
        let present_queue = unsafe {
            self.device
                .get_device_queue(0, self.queue_families.presentation_index as u32)
        };
        let mut current_frame: usize = 0;
        while !(self.window.should_close()) {
            unsafe { glfw::ffi::glfwPollEvents() };
            window::draw_frame(
                &self.sync[current_frame],
                &self.device,
                &self.swapchain_device,
                self.swapchain,
                self.command_buffer[current_frame],
                self.render_pass,
                &self.framebuffers,
                self.extent,
                self.graphics_pipeline,
                graphics_queue,
                present_queue,
            );
            current_frame = (current_frame + 1) % (MAX_FRAMES_IN_FLIGHT as usize);
        }
    }
}

fn create_vk_instance(
    entry: &Entry,
    required_instance_extensions: Option<Vec<String>>,
) -> Instance {
    let app_name = CString::new("Mobject").unwrap();
    let engine_name = CString::new("No Engine").unwrap();

    let required_instance_extensions = required_instance_extensions.unwrap_or_default();
    let app_info = ApplicationInfo {
        s_type: StructureType::APPLICATION_INFO,
        p_application_name: app_name.as_ptr(),
        application_version: vk::make_api_version(1, 0, 0, 0),
        p_engine_name: engine_name.as_ptr(),
        engine_version: vk::make_api_version(1, 1, 1, 0),
        api_version: vk::make_api_version(0, 1, 1, 0),
        ..Default::default()
    };

    #[cfg(feature = "validation_layers")]
    let utf8_ptr = Utf8Pointer::new(&VALIDATION_LAYERS);
    let extensions_ptr = Utf8Pointer::new(&required_instance_extensions);
    let instance_info = InstanceCreateInfo {
        s_type: StructureType::INSTANCE_CREATE_INFO,
        p_application_info: &app_info,
        #[cfg(feature = "validation_layers")]
        enabled_layer_count: 1,
        #[cfg(feature = "validation_layers")]
        pp_enabled_layer_names: utf8_ptr.as_ptr(),
        enabled_extension_count: required_instance_extensions.len() as u32,
        pp_enabled_extension_names: extensions_ptr.as_ptr(),
        ..Default::default()
    };

    unsafe { entry.create_instance(&instance_info, None).unwrap() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoketest_it_works() {
        let entry = unsafe { Entry::load().unwrap() };
        create_vk_instance(&entry, None);
    }
}
