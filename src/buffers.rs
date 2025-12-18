use std::char::MAX;
use std::ffi::c_void;

use crate::MAX_FRAMES_IN_FLIGHT;
use crate::device::{self, QueueFamilies};
use crate::shapes::{Shape, UBO, Vertex2D};
use ash::Device;
use ash::vk::{
    self, Buffer, BufferCreateInfo, BufferUsageFlags, ClearColorValue, ClearValue, CommandBuffer,
    CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandPool, CommandPoolCreateInfo,
    DescriptorSet, DeviceMemory, DeviceSize, Extent2D, Framebuffer, FramebufferCreateInfo, Handle,
    ImageView, MemoryAllocateInfo, MemoryMapFlags, MemoryPropertyFlags, MemoryRequirements,
    Offset2D, PhysicalDevice, PhysicalDeviceMemoryProperties, Pipeline, PipelineBindPoint,
    PipelineLayout, Rect2D, RenderPass, RenderPassBeginInfo, StructureType, SubpassContents,
};

pub fn create_frame_buffers(
    device: &Device,
    render_pass: RenderPass,
    image_views: &[ImageView],
    extent: Extent2D,
) -> Vec<Framebuffer> {
    let framebuffer_infos: Vec<FramebufferCreateInfo> = image_views
        .iter()
        .map(|view| FramebufferCreateInfo {
            s_type: StructureType::FRAMEBUFFER_CREATE_INFO,
            attachment_count: 1,
            p_attachments: view as *const ImageView,
            width: extent.width,
            height: extent.height,
            layers: 1,
            render_pass,
            ..Default::default()
        })
        .collect();

    framebuffer_infos
        .iter()
        .map(|info| unsafe { device.create_framebuffer(info, None) }.unwrap())
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

pub fn create_command_buffers(pool: CommandPool, device: &Device) -> Vec<CommandBuffer> {
    let create_info = CommandBufferAllocateInfo {
        s_type: StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        command_pool: pool,
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: MAX_FRAMES_IN_FLIGHT,
        ..Default::default()
    };
    // taking the first one since we only created one buffer
    // NOTE: rust takes just a single reference for create_info, the C++ API takes a pointer to
    // a vec
    unsafe { device.allocate_command_buffers(&create_info) }.unwrap()
}

pub fn record_command_buffer(
    device: &Device,
    buffer: CommandBuffer,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    image_index: u32,
    render_pass: RenderPass,
    framebuffers: &[Framebuffer],
    descriptor_set: DescriptorSet,
    extent: Extent2D,
    pipeline_layout: PipelineLayout,
    graphics_pipeline: Pipeline,
) {
    let command_begin_info = CommandBufferBeginInfo {
        s_type: StructureType::COMMAND_BUFFER_BEGIN_INFO,
        ..Default::default()
    };
    unsafe { device.begin_command_buffer(buffer, &command_begin_info) }.unwrap();
    let clear_color = ClearValue {
        color: ClearColorValue {
            float32: [0.0, 0.0, 0.0, 0.0],
        },
    };
    let render_pass_begin_info = RenderPassBeginInfo {
        s_type: StructureType::RENDER_PASS_BEGIN_INFO,
        render_pass,
        framebuffer: framebuffers[image_index as usize],
        render_area: Rect2D {
            offset: Offset2D { x: 0, y: 0 },
            extent,
        },
        clear_value_count: 1,
        p_clear_values: &clear_color,
        ..Default::default()
    };
    unsafe {
        device.cmd_begin_render_pass(buffer, &render_pass_begin_info, SubpassContents::INLINE)
    };

    unsafe { device.cmd_bind_pipeline(buffer, vk::PipelineBindPoint::GRAPHICS, graphics_pipeline) };

    unsafe {
        device.cmd_bind_vertex_buffers(buffer, 0, &[vertex_buffer], &[0]);
    }
    unsafe { device.cmd_bind_index_buffer(buffer, index_buffer, 0, vk::IndexType::UINT32) };
    unsafe {
        device.cmd_bind_descriptor_sets(
            buffer,
            PipelineBindPoint::GRAPHICS,
            pipeline_layout,
            0,
            &[descriptor_set],
            &[],
        )
    };
    unsafe { device.cmd_draw_indexed(buffer, num_indices, 1, 0, 0, 0) };

    unsafe { device.cmd_end_render_pass(buffer) };
    // end recording command buffer: not necassarily finishing the execution
    unsafe { device.end_command_buffer(buffer) }.unwrap();
}

pub fn create_vertex_buffers(
    device: &Device,
    num_vertices_upper_bound: usize,
    memory_proprties: PhysicalDeviceMemoryProperties,
) -> (Vec<Buffer>, Vec<DeviceMemory>) {
    let entities: Vec<(Buffer, DeviceMemory)> = (0..MAX_FRAMES_IN_FLIGHT)
        .map(|_| {
            create_buffer(
                device,
                (size_of::<Vertex2D>() * num_vertices_upper_bound) as u64,
                BufferUsageFlags::VERTEX_BUFFER,
                memory_proprties,
            )
        })
        .collect();
    let buffers = entities.iter().map(|(buffer, _)| *buffer).collect();
    let memories = entities.iter().map(|(_, memory)| *memory).collect();
    (buffers, memories)
}

pub fn create_index_buffers(
    device: &Device,
    num_indices_upper_bound: usize,
    memory_proprties: PhysicalDeviceMemoryProperties,
) -> (Vec<Buffer>, Vec<DeviceMemory>) {
    let entities: Vec<(Buffer, DeviceMemory)> = (0..MAX_FRAMES_IN_FLIGHT)
        .map(|_| {
            create_buffer(
                device,
                (size_of::<u32>() * num_indices_upper_bound) as u64,
                BufferUsageFlags::INDEX_BUFFER,
                memory_proprties,
            )
        })
        .collect();
    let buffers = entities.iter().map(|(buffer, _)| *buffer).collect();
    let memories = entities.iter().map(|(_, memory)| *memory).collect();
    (buffers, memories)
}

pub fn create_uniform_buffers(
    device: &Device,
    memory_proprties: PhysicalDeviceMemoryProperties,
) -> (Vec<Buffer>, Vec<DeviceMemory>, Vec<*mut c_void>) {
    let size = size_of::<UBO>() as u64;
    let entities: Vec<(Buffer, DeviceMemory)> = (0..MAX_FRAMES_IN_FLIGHT)
        .map(|_| {
            create_buffer(
                device,
                size,
                BufferUsageFlags::UNIFORM_BUFFER,
                memory_proprties,
            )
        })
        .collect();
    let buffers = entities.iter().map(|(buffer, _)| *buffer).collect();
    let memories: Vec<DeviceMemory> = entities.iter().map(|(_, memory)| *memory).collect();
    let mapped_memories: Vec<*mut c_void> = memories
        .iter()
        .map(|&mem| unsafe { device.map_memory(mem, 0, size, MemoryMapFlags::empty()) }.unwrap())
        .collect();
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

fn find_memory_type(
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
