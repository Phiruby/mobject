pub mod c_utils;
pub mod device;
pub mod render_pass;
pub mod shaders;
pub mod swapchain;
pub mod window;

use ash::vk::{self, ApplicationInfo, Handle, InstanceCreateInfo, StructureType, SurfaceKHR};
use ash::{Entry, Instance, khr, khr::surface};
use c_utils::Utf8Pointer;
use device::QueueFamilies;
use std::ffi::CString;
use swapchain::SwapchainSupport;
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
const DEVICE_EXTENSIONS: [&str; 1] = ["VK_KHR_swapchain"];
pub struct Scene {}

impl Scene {
    pub fn new() {
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
        let extent = swapchain_capabilities.choose_extent(window);
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
        let swapchain_imageviews =
            swapchain::create_image_views(&logical_device, &swapchain_images, surface_format);

        let render_pass = render_pass::create(surface_format, &logical_device);
        let graphics_pipeline =
            shaders::create_graphics_pipeline(&logical_device, extent, render_pass);
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
