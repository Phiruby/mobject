use crate::buffers;
use ash::Device;
use ash::vk::{Buffer, BufferUsageFlags, DeviceMemory, PhysicalDeviceMemoryProperties};
use image::{DynamicImage, ImageBuffer, ImageReader, Rgba32FImage};
pub fn load_image(path: &str) -> Rgba32FImage {
    ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .as_rgba32f()
        .unwrap()
        .clone() // NOTE: forced to clone since returns a reference to rgba
}

pub fn create_texture_image(
    device: &Device,
    image_path: &str,
    physical_device_memory_properties: PhysicalDeviceMemoryProperties,
) {
    let pixels = load_image(image_path);
    let (width, height) = pixels.dimensions();
    let size = width * height * 4; // 4 bytes = 32 bits (RGBA f32)
    let buffer = buffers::create_buffer(
        device,
        size as u64,
        BufferUsageFlags::TRANSFER_SRC,
        physical_device_memory_properties,
    );
}
