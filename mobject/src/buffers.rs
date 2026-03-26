use std::array;
use std::char::MAX;
use std::ffi::c_void;

use crate::device::{self, QueueFamilies};
use crate::shapes::{BuiltShape, GlobalUBO, Shape, UBO, Vertex2D};
use crate::{MAX_FRAMES_IN_FLIGHT, swapchain, texture};
use ash::vk::{
    self, Buffer, BufferCreateInfo, BufferUsageFlags, ClearColorValue, ClearDepthStencilValue,
    ClearValue, CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandPool,
    CommandPoolCreateInfo, DescriptorSet, DeviceMemory, DeviceSize, Extent2D, Fence, Format,
    FormatFeatureFlags, Framebuffer, FramebufferCreateInfo, Handle, Image, ImageAspectFlags,
    ImageTiling, ImageUsageFlags, ImageView, MemoryAllocateInfo, MemoryMapFlags,
    MemoryPropertyFlags, MemoryRequirements, Offset2D, PhysicalDevice,
    PhysicalDeviceMemoryProperties, Pipeline, PipelineBindPoint, PipelineLayout, Queue, Rect2D,
    RenderPass, RenderPassBeginInfo, StructureType, SubmitInfo, SubpassContents,
};
use ash::{Device, Instance};

pub fn create_frame_buffers(
    device: &Device,
    render_pass: RenderPass,
    swapchain_image_views: &[ImageView],
    depth_image: ImageView,
    extent: Extent2D,
) -> Vec<Framebuffer> {
    // preserving attachments until `create_frame_buffers` is called
    let attachments: Vec<[ImageView; 2]> = swapchain_image_views
        .iter()
        .map(|img| [*img, depth_image])
        .collect();
    let framebuffer_infos: Vec<FramebufferCreateInfo> = attachments
        .iter()
        .map(|attachments| FramebufferCreateInfo {
            s_type: StructureType::FRAMEBUFFER_CREATE_INFO,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: extent.width,
            height: extent.height,
            layers: 1,
            render_pass,
            ..Default::default()
        })
        .collect();

    framebuffer_infos
        .iter()
        .map(|(info)| unsafe { device.create_framebuffer(info, None) }.unwrap())
        .collect::<Vec<Framebuffer>>()
}

pub fn create_command_pool(device: &Device, queue_families: &QueueFamilies) -> CommandPool {
    let create_info = CommandPoolCreateInfo {
        s_type: StructureType::COMMAND_POOL_CREATE_INFO,
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_family_index: queue_families.graphics_index as u32,
        ..Default::default()
    };
    unsafe { device.create_command_pool(&create_info, None) }.unwrap()
}

pub fn create_command_buffers(pool: CommandPool, device: &Device, num: u32) -> Vec<CommandBuffer> {
    let create_info = CommandBufferAllocateInfo {
        s_type: StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        command_pool: pool,
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: num,
        ..Default::default()
    };
    // taking the first one since we only created one buffer
    // NOTE: rust takes just a single reference for create_info, the C++ API takes a pointer to
    // a vec
    unsafe { device.allocate_command_buffers(&create_info) }.unwrap()
}

pub fn begin_single_time_recording(pool: CommandPool, device: &Device) -> CommandBuffer {
    let command_buffer = create_command_buffers(pool, device, 1)[0];
    let begin_info = CommandBufferBeginInfo {
        s_type: StructureType::COMMAND_BUFFER_BEGIN_INFO,
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
    };
    unsafe { device.begin_command_buffer(command_buffer, &begin_info) }.unwrap();
    command_buffer
}

pub fn end_single_time_recording(
    device: &Device,
    buffer: CommandBuffer,
    queue: Queue,
    pool: CommandPool,
) {
    unsafe { device.end_command_buffer(buffer) }.unwrap();
    let submit_info = SubmitInfo {
        s_type: StructureType::SUBMIT_INFO,
        command_buffer_count: 1,
        p_command_buffers: &buffer,
        ..Default::default()
    };
    unsafe { device.queue_submit(queue, &[submit_info], Fence::null()) }.unwrap();
    unsafe { device.queue_wait_idle(queue) }.unwrap();
    unsafe { device.free_command_buffers(pool, &[buffer]) };
}

pub fn create_vertex_buffers(
    device: &Device,
    num_vertices_upper_bound: usize,
    memory_proprties: PhysicalDeviceMemoryProperties,
) -> ([Buffer; MAX_FRAMES_IN_FLIGHT as usize], [DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]) {
    let entities: [(Buffer, DeviceMemory); MAX_FRAMES_IN_FLIGHT as usize] = array::from_fn(|_| {
            create_buffer(
                device,
                (size_of::<Vertex2D>() * num_vertices_upper_bound) as u64,
                BufferUsageFlags::VERTEX_BUFFER,
                memory_proprties,
            )
        });
    let buffers = array::from_fn(|i| entities[i].0);
    let memories = array::from_fn(|i| entities[i].1);
    (buffers, memories)
}

pub fn create_index_buffers(
    device: &Device,
    num_indices_upper_bound: usize,
    memory_proprties: PhysicalDeviceMemoryProperties,
) -> ([Buffer; MAX_FRAMES_IN_FLIGHT as usize], [DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]) {
    let entities: [(Buffer, DeviceMemory); MAX_FRAMES_IN_FLIGHT as usize] = array::from_fn(|_| {
            create_buffer(
                device,
                (size_of::<u32>() * num_indices_upper_bound) as u64,
                BufferUsageFlags::INDEX_BUFFER,
                memory_proprties,
            )
        });

    let buffers = array::from_fn(|i| entities[i].0);
    let memories = array::from_fn(|i| entities[i].1);
    (buffers, memories)
}

pub fn create_uniform_buffers<const N: usize>(
    device: &Device,
    memory_proprties: PhysicalDeviceMemoryProperties,
    struct_size: u64,
) -> ([Buffer; N], [DeviceMemory; N], [*mut c_void; N]) {
    let entities: [(Buffer, DeviceMemory); N] = array::from_fn(|_| {
        create_buffer(
            device,
            struct_size,
            BufferUsageFlags::UNIFORM_BUFFER,
            memory_proprties,
        )
    });
    let buffers: [Buffer; N] = array::from_fn(|i| entities[i].0);
    let memories: [DeviceMemory; N] = array::from_fn(|i| entities[i].1);
    let mapped_memories: [*mut c_void; N] = array::from_fn(|i| unsafe {
        device
            .map_memory(memories[i], 0, struct_size, MemoryMapFlags::empty())
            .unwrap()
    });
    (buffers, memories, mapped_memories)
}

pub fn create_buffer(
    device: &Device,
    size: DeviceSize,
    usage: BufferUsageFlags,
    memory_proprties: PhysicalDeviceMemoryProperties,
) -> (Buffer, DeviceMemory) {
    let create_info = BufferCreateInfo {
        s_type: StructureType::BUFFER_CREATE_INFO,
        size: size,
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };
    let buffer = unsafe { device.create_buffer(&create_info, None) }.unwrap();
    let memory = allocate_vertex_buffers_memory(
        &[buffer],
        device,
        memory_proprties,
        MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
    );
    (buffer, memory[0])
}

pub fn find_memory_type(
    memory_proprties: PhysicalDeviceMemoryProperties,
    type_filter: u32,
    properties: MemoryPropertyFlags,
) -> u32 {
    (0..memory_proprties.memory_type_count)
        .find(|i| {
            ((type_filter & (1 << i)) != 0)
                && ((memory_proprties.memory_types[*i as usize].property_flags & properties)
                    == properties)
        })
        .unwrap()
}

fn find_depth_buffer_format(
    instance: &Instance,
    physical_device: PhysicalDevice,
    candidates: Vec<Format>,
    tiling: ImageTiling,
    features: FormatFeatureFlags,
) -> Format {
    candidates
        .into_iter()
        .find(|&candidate| {
            let props = unsafe {
                instance.get_physical_device_format_properties(physical_device, candidate)
            };
            ((tiling == ImageTiling::LINEAR && (props.linear_tiling_features.intersects(features)))
                || (tiling == ImageTiling::OPTIMAL
                    && (props.optimal_tiling_features.intersects(features))))
        })
        .expect("Could not find a format for depth buffer")
}

pub fn create_depth_buffer(
    instance: &Instance,
    logical_device: &Device,
    physical_device: PhysicalDevice,
    extent: Extent2D,
    physical_device_memory_properties: PhysicalDeviceMemoryProperties,
) -> (Image, ImageView, DeviceMemory, Format) {
    let depth_format = find_depth_buffer_format(
        instance,
        physical_device,
        vec![
            Format::D32_SFLOAT,
            Format::D32_SFLOAT_S8_UINT,
            Format::D24_UNORM_S8_UINT,
        ],
        ImageTiling::OPTIMAL,
        FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    );
    let (image, image_memory) = texture::create_image(
        logical_device,
        extent.width,
        extent.height,
        1,
        depth_format,
        ImageTiling::OPTIMAL,
        ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        MemoryPropertyFlags::DEVICE_LOCAL,
        physical_device_memory_properties,
    );
    let image_view = swapchain::create_image_views(
        logical_device,
        &[image],
        1,
        depth_format,
        ImageAspectFlags::DEPTH,
    )[0];
    (image, image_view, image_memory, depth_format)
}

fn allocate_vertex_buffers_memory(
    buffers: &[Buffer],
    device: &Device,
    physical_device_memory_properties: PhysicalDeviceMemoryProperties,
    properties: MemoryPropertyFlags,
) -> Vec<DeviceMemory> {
    let requirements: Vec<MemoryRequirements> = buffers
        .iter()
        .copied()
        .map(|buffer| unsafe { device.get_buffer_memory_requirements(buffer) })
        .collect();
    let alloc_infos: Vec<MemoryAllocateInfo> = requirements
        .iter()
        .map(|requirement| MemoryAllocateInfo {
            allocation_size: requirement.size,
            memory_type_index: find_memory_type(
                physical_device_memory_properties,
                requirement.memory_type_bits,
                properties,
            ),
            ..Default::default()
        })
        .collect();

    let memories: Vec<DeviceMemory> = alloc_infos
        .iter()
        .map(|info| unsafe { device.allocate_memory(info, None) }.unwrap())
        .collect();
    buffers
        .iter()
        .zip(memories.iter())
        .for_each(|(&buffer, &memory)| {
            unsafe { device.bind_buffer_memory(buffer, memory, 0) }.unwrap()
        });
    memories
}
