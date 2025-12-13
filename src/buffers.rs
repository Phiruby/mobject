use crate::device::{self, QueueFamilies};
use ash::Device;
use ash::vk::{
    self, ClearColorValue, ClearValue, CommandBuffer, CommandBufferAllocateInfo,
    CommandBufferBeginInfo, CommandPool, CommandPoolCreateInfo, Extent2D, Framebuffer,
    FramebufferCreateInfo, Handle, ImageView, Offset2D, Pipeline, Rect2D, RenderPass,
    RenderPassBeginInfo, StructureType, SubpassContents,
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

pub fn create_command_buffer(pool: CommandPool, device: &Device) -> CommandBuffer {
    let create_info = CommandBufferAllocateInfo {
        s_type: StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        command_pool: pool,
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: 1,
        ..Default::default()
    };
    // taking the first one since we only created one buffer
    // NOTE: rust takes just a single reference for create_info, the C++ API takes a pointer to
    // a vec
    unsafe { device.allocate_command_buffers(&create_info) }.unwrap()[0]
}

pub fn record_command_buffer(
    device: &Device,
    buffer: CommandBuffer,
    image_index: u32,
    render_pass: RenderPass,
    framebuffers: &[Framebuffer],
    extent: Extent2D,
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

    unsafe { device.cmd_draw(buffer, 3, 1, 0, 0) };
    // end recording command buffer: not necassarily finishing the execution
    unsafe { device.end_command_buffer(buffer) }.unwrap();
}
