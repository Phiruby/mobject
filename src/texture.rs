use crate::buffers;
use crate::swapchain;
use ash::vk::ImageBlit;
use ash::vk::Sampler;
use ash::vk::SamplerCreateInfo;
use ash::vk::{
    self, AccessFlags, Buffer, BufferImageCopy, BufferUsageFlags, CommandPool, DependencyFlags,
    DeviceMemory, Extent2D, Extent3D, Format, Image, ImageAspectFlags, ImageCreateFlags,
    ImageCreateInfo, ImageLayout, ImageMemoryBarrier, ImageSubresource, ImageSubresourceLayers,
    ImageSubresourceRange, ImageTiling, ImageUsageFlags, ImageView, ImageViewCreateInfo,
    ImageViewType, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, Offset3D,
    PhysicalDevice, PhysicalDeviceMemoryProperties, PipelineStageFlags, Queue, StructureType,
};
use ash::{Device, Instance};
use image::{DynamicImage, ImageBuffer, ImageReader, RgbaImage};
fn load_image(path: &str) -> RgbaImage {
    ImageReader::open(path)
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8()
}

pub fn create_image(
    device: &Device,
    width: u32,
    height: u32,
    mip_levels: u32,
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
        mip_levels,
        format,
        tiling,
        initial_layout: vk::ImageLayout::UNDEFINED,
        usage: image_usage,
        samples: vk::SampleCountFlags::TYPE_1,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        array_layers: 1,
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

fn transition_image_layout(
    device: &Device,
    pool: vk::CommandPool,
    image: Image,
    mip_levels: u32,
    format: Format,
    old_layout: ImageLayout,
    new_layout: ImageLayout,
    graphics_queue: Queue,
) {
    let cmd_buffer = buffers::begin_single_time_recording(pool, device);

    let mut barrier = ImageMemoryBarrier {
        s_type: StructureType::IMAGE_MEMORY_BARRIER,
        old_layout,
        new_layout,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image,
        subresource_range: ImageSubresourceRange {
            aspect_mask: ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: 1,
        },
        src_access_mask: AccessFlags::empty(),
        dst_access_mask: AccessFlags::empty(),
        ..Default::default()
    };

    let mut source_stage;
    let mut destination_stage;
    match (old_layout, new_layout) {
        (ImageLayout::UNDEFINED, ImageLayout::TRANSFER_DST_OPTIMAL) => {
            barrier.src_access_mask = AccessFlags::empty();
            barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            source_stage = PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = PipelineStageFlags::TRANSFER;
        }
        (ImageLayout::TRANSFER_DST_OPTIMAL, ImageLayout::SHADER_READ_ONLY_OPTIMAL) => {
            barrier.src_access_mask = AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = AccessFlags::SHADER_READ;
            source_stage = PipelineStageFlags::TRANSFER;
            destination_stage = PipelineStageFlags::FRAGMENT_SHADER;
        }
        _ => panic!("Unsupported image transition"),
    }
    unsafe {
        device.cmd_pipeline_barrier(
            cmd_buffer,
            source_stage,
            destination_stage,
            DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    };
    buffers::end_single_time_recording(device, cmd_buffer, graphics_queue, pool);
}

fn copy_buffer_to_image(
    device: &Device,
    pool: CommandPool,
    buffer: Buffer,
    image: Image,
    width: u32,
    height: u32,
    graphics_queue: Queue,
) {
    let cmd_buffer = buffers::begin_single_time_recording(pool, device);

    let region = BufferImageCopy {
        buffer_offset: 0,
        buffer_row_length: 0,
        buffer_image_height: 0,
        image_subresource: ImageSubresourceLayers {
            aspect_mask: ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        },
        image_offset: Offset3D { x: 0, y: 0, z: 0 },
        image_extent: Extent3D {
            width,
            height,
            depth: 1,
        },
    };
    unsafe {
        device.cmd_copy_buffer_to_image(
            cmd_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        )
    };

    buffers::end_single_time_recording(device, cmd_buffer, graphics_queue, pool);
}

pub fn create_texture_image(
    device: &Device,
    image_path: &str,
    physical_device_memory_properties: PhysicalDeviceMemoryProperties,
    pool: CommandPool,
    graphics_queue: Queue,
) -> (Image, DeviceMemory, u32) {
    let pixels = load_image(image_path);
    let (width, height) = pixels.dimensions();
    let mip_levels = std::cmp::max(width, height).ilog2() + 1;
    let size = width * height * 4; // 4 channels; one byte each
    let (buffer, memory) = buffers::create_buffer(
        device,
        size as u64,
        BufferUsageFlags::TRANSFER_SRC,
        physical_device_memory_properties,
    );
    let data_loc =
        unsafe { device.map_memory(memory, 0, size as u64, MemoryMapFlags::empty()) }.unwrap();
    let raw_pixels = pixels.as_raw();
    unsafe {
        std::ptr::copy_nonoverlapping(raw_pixels.as_ptr(), data_loc as *mut u8, size as usize)
    };
    unsafe { device.unmap_memory(memory) };
    let (image, image_memory) = create_image(
        device,
        width,
        height,
        mip_levels,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::SAMPLED,
        MemoryPropertyFlags::DEVICE_LOCAL,
        physical_device_memory_properties,
    );
    transition_image_layout(
        device,
        pool,
        image,
        mip_levels,
        Format::R8G8B8A8_SRGB,
        ImageLayout::UNDEFINED,
        ImageLayout::TRANSFER_DST_OPTIMAL,
        graphics_queue,
    );
    copy_buffer_to_image(device, pool, buffer, image, width, height, graphics_queue);
    generate_mipmaps(
        pool,
        device,
        image,
        graphics_queue,
        width,
        height,
        mip_levels,
    );
    // transition_image_layout(
    //     device,
    //     pool,
    //     image,
    //     mip_levels,
    //     Format::R8G8B8A8_SRGB,
    //     ImageLayout::TRANSFER_DST_OPTIMAL,
    //     ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    //     graphics_queue,
    // );
    unsafe { device.destroy_buffer(buffer, None) };
    unsafe { device.free_memory(memory, None) };
    (image, image_memory, mip_levels)
}

pub fn create_texture_image_view(device: &Device, image: Image, mip_levels: u32) -> ImageView {
    // taking the first since only one image created
    swapchain::create_image_views(
        device,
        &[image],
        mip_levels,
        Format::R8G8B8A8_SRGB,
        ImageAspectFlags::COLOR,
    )[0]
}

pub fn create_sampler(
    logical_device: &Device,
    instance: &Instance,
    physical_device: PhysicalDevice,
) -> Sampler {
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    let sampler_info = SamplerCreateInfo {
        s_type: StructureType::SAMPLER_CREATE_INFO,
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        address_mode_u: vk::SamplerAddressMode::REPEAT,
        address_mode_v: vk::SamplerAddressMode::REPEAT,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        anisotropy_enable: vk::TRUE,
        max_anisotropy: properties.limits.max_sampler_anisotropy,
        border_color: vk::BorderColor::INT_OPAQUE_BLACK,
        unnormalized_coordinates: vk::FALSE,
        compare_enable: vk::FALSE,
        compare_op: vk::CompareOp::ALWAYS,
        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
        mip_lod_bias: 0.0,
        min_lod: 0.0,
        max_lod: 0.0,
        ..Default::default()
    };

    unsafe { logical_device.create_sampler(&sampler_info, None) }.unwrap()
}

fn generate_mipmaps(
    pool: CommandPool,
    device: &Device,
    image: Image,
    graphics_queue: Queue,
    tex_width: u32,
    tex_height: u32,
    mip_levels: u32,
) {
    let command_buffer = buffers::begin_single_time_recording(pool, device);

    let mut barrier = ImageMemoryBarrier {
        s_type: StructureType::IMAGE_MEMORY_BARRIER,
        image,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        subresource_range: ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut mip_width = tex_width;
    let mut mip_height = tex_height;
    for i in (1..mip_levels) {
        barrier.subresource_range.base_mip_level = i - 1;
        barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        };
        let blit = ImageBlit {
            src_offsets: [
                Offset3D { x: 0, y: 0, z: 0 },
                Offset3D {
                    x: mip_width as i32,
                    y: mip_height as i32,
                    z: 1,
                },
            ],
            src_subresource: ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: i - 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            dst_offsets: [
                Offset3D { x: 0, y: 0, z: 0 },
                Offset3D {
                    x: if mip_width > 1 { mip_width / 2 } else { 1 } as i32,
                    y: if mip_height > 1 { mip_height / 2 } else { 1 } as i32,
                    z: 1,
                },
            ],
            dst_subresource: ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: i,
                base_array_layer: 0,
                layer_count: 1,
            },
        };

        unsafe {
            device.cmd_blit_image(
                command_buffer,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[blit],
                vk::Filter::LINEAR,
            );
        };

        barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };

        if (mip_height > 1) {
            mip_height = mip_height / 2;
        }
        if (mip_width > 1) {
            mip_width = mip_width / 2;
        }
    }

    barrier.subresource_range.base_mip_level = mip_levels - 1;
    barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
    barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        )
    };

    buffers::end_single_time_recording(device, command_buffer, graphics_queue, pool);
}
