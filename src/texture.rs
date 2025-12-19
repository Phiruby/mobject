use crate::buffers;
use ash::Device;
use ash::vk::{
    self, Buffer, BufferUsageFlags, DeviceMemory, Extent2D, Extent3D, Format, Image,
    ImageCreateFlags, ImageCreateInfo, ImageTiling, ImageUsageFlags, MemoryAllocateInfo,
    MemoryMapFlags, MemoryPropertyFlags, PhysicalDeviceMemoryProperties, StructureType,
};
use image::{DynamicImage, ImageBuffer, ImageReader, Rgba32FImage};
fn load_image(path: &str) -> Rgba32FImage {
    ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .as_rgba32f()
        .unwrap()
        .clone() // NOTE: forced to clone since returns a reference to rgba
}

fn create_image(
    device: &Device,
    width: u32,
    height: u32,
    format: Format,
    tiling: ImageTiling,
    image_usage: ImageUsageFlags,
    properties: MemoryPropertyFlags,
    physical_device_memory_properties: PhysicalDeviceMemoryProperties,
) -> (Image, DeviceMemory) {
    let image_info = ImageCreateInfo {
        s_type: StructureType::IMAGE_CREATE_INFO,
        image_type: vk::ImageType::TYPE_2D,
        extent: Extent3D {
            width,
            height,
            depth: 1,
        },
        mip_levels: 1,
        format,
        tiling,
        initial_layout: vk::ImageLayout::UNDEFINED,
        usage: image_usage,
        samples: vk::SampleCountFlags::TYPE_1,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };
    let image = unsafe { device.create_image(&image_info, None) }.unwrap();
    let image_mem_requirements = unsafe { device.get_image_memory_requirements(image) };
    let alloc_info = MemoryAllocateInfo {
        s_type: StructureType::MEMORY_ALLOCATE_INFO,
        allocation_size: image_mem_requirements.size,
        memory_type_index: buffers::find_memory_type(
            physical_device_memory_properties,
            image_mem_requirements.memory_type_bits,
            properties,
        ),
        ..Default::default()
    };
    let image_memory = unsafe { device.allocate_memory(&alloc_info, None) }.unwrap();
    unsafe { device.bind_image_memory(image, image_memory, 0) }.unwrap();
    (image, image_memory)
}

pub fn create_texture_image(
    device: &Device,
    image_path: &str,
    physical_device_memory_properties: PhysicalDeviceMemoryProperties,
) -> (Buffer, DeviceMemory, Image, DeviceMemory) {
    let pixels = load_image(image_path);
    let (width, height) = pixels.dimensions();
    let size = width * height * 4; // 4 bytes = 32 bits (RGBA f32)
    let (buffer, memory) = buffers::create_buffer(
        device,
        size as u64,
        BufferUsageFlags::TRANSFER_SRC,
        physical_device_memory_properties,
    );
    let data_loc =
        unsafe { device.map_memory(memory, 0, size as u64, MemoryMapFlags::empty()) }.unwrap();
    unsafe {
        std::ptr::copy_nonoverlapping(
            pixels.as_raw().as_ptr(),
            data_loc as *mut f32,
            size as usize,
        )
    };
    unsafe { device.unmap_memory(memory) };
    let (image, image_memory) = create_image(
        device,
        width,
        height,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        MemoryPropertyFlags::DEVICE_LOCAL,
        physical_device_memory_properties,
    );
    (buffer, memory, image, image_memory)
}
