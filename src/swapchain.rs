use ash::vk::{
    self, ColorSpaceKHR, Extent2D, Format, PhysicalDevice, PresentModeKHR, SurfaceCapabilitiesKHR,
    SurfaceFormatKHR, SurfaceKHR,
};
use ash::{Entry, Instance, khr};
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
    pub fn choose_surface_format(&mut self) -> &SurfaceFormatKHR {
        self.chosen_format.get_or_insert_with(|| {
            let format = self
                .formats
                .iter()
                .find(|format| {
                    format.format == Format::B8G8R8A8_SRGB
                        && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
                })
                .expect("No surface format with desired format and color space available");
            *format
        })
    }

    pub fn choose_present_mode(&mut self) -> &PresentModeKHR {
        self.chosen_present_mode.get_or_insert_with(|| {
            let present_modes = self
                .present_modes
                .iter()
                .find(|mode| **mode == vk::PresentModeKHR::MAILBOX)
                .copied();
            present_modes.unwrap_or(PresentModeKHR::FIFO)
        })
    }

    pub fn choose_extent(&mut self, window: PWindow) -> Extent2D {
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
