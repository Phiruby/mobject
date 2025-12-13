use ash::Device;
use ash::vk::{
    self, AccessFlags, AttachmentDescription, AttachmentReference, PipelineBindPoint, RenderPass,
    RenderPassCreateInfo, StructureType, SubpassDependency, SubpassDescription, SurfaceFormatKHR,
};

pub fn create(format: SurfaceFormatKHR, device: &Device) -> RenderPass {
    // fragment shader will only output color (layout 0 is color); so just color attachment for now
    let color_attachment = AttachmentDescription {
        format: format.format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    };

    let color_attachment_ref = AttachmentReference {
        attachment: 0, // we only have 1 attachment (above); so hard stencil_load_op
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, // transition to this layout in this
                       // subpass
    };

    let subpass = SubpassDescription {
        pipeline_bind_point: PipelineBindPoint::GRAPHICS,
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        ..Default::default()
    };
    // describe transition of memory layout: implicit step at the beginning to create initial
    // layout
    let subpass_dependency = SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL, // implicit: src before render pass
        dst_subpass: 0,                    // idx to out only subpass
        // waiting on these
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        src_access_mask: AccessFlags::empty(),
        // operations to wait are color attachment writing
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        ..Default::default()
    };
    let render_pass_info = RenderPassCreateInfo {
        s_type: StructureType::RENDER_PASS_CREATE_INFO,
        attachment_count: 1,
        p_attachments: &color_attachment,
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: 1,
        p_dependencies: &subpass_dependency,
        ..Default::default()
    };

    unsafe { device.create_render_pass(&render_pass_info, None) }.unwrap()
}
