use crate::device::QueueFamilies;
use ash::vk::{
    self, ColorSpaceKHR, ComponentMapping, ComponentSwizzle, CompositeAlphaFlagsKHR, Extent2D,
    Format, Image, ImageAspectFlags, ImageCreateInfo, ImageSubresourceRange, ImageView,
    ImageViewCreateInfo, ImageViewType, PhysicalDevice, PresentModeKHR, SharingMode, StructureType,
    SurfaceCapabilitiesKHR, SurfaceFormatKHR, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR,
};
use ash::{Device, Entry, Instance, khr};
use glfw::PWindow;

pub struct SwapchainSupport {
    pub capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
    chosen_format: Option<SurfaceFormatKHR>,
    chosen_present_mode: Option<PresentModeKHR>,
    extent: Option<Extent2D>,
}

pub fn query_support(
    device: PhysicalDevice,
    surface_instance: &khr::surface::Instance,
    surface: SurfaceKHR,
) -> SwapchainSupport {
    let capabilities =
        unsafe { surface_instance.get_physical_device_surface_capabilities(device, surface) }
            .unwrap();
    let formats =
        unsafe { surface_instance.get_physical_device_surface_formats(device, surface) }.unwrap();
    let present_modes =
        unsafe { surface_instance.get_physical_device_surface_present_modes(device, surface) }
            .unwrap();

    SwapchainSupport {
        capabilities,
        formats,
        present_modes,
        chosen_format: None,
        chosen_present_mode: None,
        extent: None,
    }
}

impl SwapchainSupport {
    pub fn choose_surface_format(&mut self) -> SurfaceFormatKHR {
        let format = self.chosen_format.get_or_insert_with(|| {
            self.formats
                .iter()
                .find(|format| {
                    format.format == Format::B8G8R8A8_SRGB
                        && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
                })
                .copied()
                .expect("No surface format with desired format and color space available")
        });
        *format
    }

    pub fn choose_present_mode(&mut self) -> PresentModeKHR {
        let p_mode = self.chosen_present_mode.get_or_insert_with(|| {
            let present_modes = self
                .present_modes
                .iter()
                .find(|mode| **mode == vk::PresentModeKHR::MAILBOX)
                .copied();
            present_modes.unwrap_or(PresentModeKHR::FIFO)
        });
        *p_mode
    }

    pub fn choose_extent(&mut self, window: &PWindow) -> Extent2D {
        match self.capabilities.current_extent.width {
            // MAX indicates that the user can choose
            u32::MAX => {
                let (width_pixels, height_pixels) = window.get_framebuffer_size();

                let min_extent = self.capabilities.min_image_extent;
                let max_extent = self.capabilities.max_image_extent;
                Extent2D {
                    width: width_pixels.clamp(min_extent.width as i32, max_extent.width as i32)
                        as u32,
                    height: height_pixels.clamp(min_extent.height as i32, max_extent.height as i32)
                        as u32,
                }
            }
            _ => self.capabilities.current_extent,
        }
    }
}

pub fn create(
    swapchain_device: &khr::swapchain::Device,
    present_mode: PresentModeKHR,
    capabilities: &SurfaceCapabilitiesKHR,
    surface: SurfaceKHR,
    format: SurfaceFormatKHR,
    queue_families: &QueueFamilies,
    extent: Extent2D,
) -> SwapchainKHR {
    let indices = vec![
        queue_families.graphics_index as u32,
        queue_families.presentation_index as u32,
    ];
    let min_image_count = match capabilities.max_image_count {
        0 => capabilities.min_image_count + 1, // 0 indicates no max
        _ => std::cmp::min(
            capabilities.min_image_count + 1,
            capabilities.max_image_count,
        ),
    };
    let mut swapchain_create_info = SwapchainCreateInfoKHR {
        s_type: StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        surface,
        present_mode,
        p_queue_family_indices: indices.as_ptr(),
        image_extent: extent,
        image_format: format.format,
        image_color_space: format.color_space,
        min_image_count,
        pre_transform: capabilities.current_transform,
        composite_alpha: CompositeAlphaFlagsKHR::OPAQUE,
        clipped: vk::TRUE,
        old_swapchain: SwapchainKHR::null(),
        image_array_layers: 1,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        ..Default::default()
    };

    if queue_families.graphics_index == queue_families.presentation_index {
        swapchain_create_info.image_sharing_mode = SharingMode::EXCLUSIVE;
        swapchain_create_info.p_queue_family_indices = std::ptr::null();
        swapchain_create_info.queue_family_index_count = 0;
    } else {
        swapchain_create_info.image_sharing_mode = SharingMode::CONCURRENT;
        swapchain_create_info.queue_family_index_count = indices.len() as u32;
        swapchain_create_info.p_queue_family_indices = indices.as_ptr();
    }

    unsafe { swapchain_device.create_swapchain(&swapchain_create_info, None) }.unwrap()
}

pub fn acquire_images(
    swapchain: SwapchainKHR,
    swapchain_device: &khr::swapchain::Device,
) -> Vec<Image> {
    unsafe { swapchain_device.get_swapchain_images(swapchain) }.unwrap()
}

pub fn create_image_views(
    logical_device: &Device,
    swapchain_images: &[Image],
    image_format: SurfaceFormatKHR,
) -> Vec<ImageView> {
    let create_infos: Vec<ImageViewCreateInfo> = swapchain_images
        .iter()
        .map(|img| ImageViewCreateInfo {
            s_type: StructureType::IMAGE_VIEW_CREATE_INFO,
            image: *img,
            view_type: ImageViewType::TYPE_2D,
            format: image_format.format,
            components: ComponentMapping {
                r: ComponentSwizzle::IDENTITY,
                g: ComponentSwizzle::IDENTITY,
                b: ComponentSwizzle::IDENTITY,
                a: ComponentSwizzle::IDENTITY,
            },
            subresource_range: ImageSubresourceRange {
                aspect_mask: ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        })
        .collect();

    create_infos
        .iter()
        .map(|info| unsafe { logical_device.create_image_view(info, None) }.unwrap())
        .collect()
}
