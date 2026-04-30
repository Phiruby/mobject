use ash::Device;
use ash::vk::{
    self, AccessFlags, AttachmentDescription, AttachmentReference, PipelineBindPoint, RenderPass, RenderPassCreateInfo,
    StructureType, SubpassDependency, SubpassDescription, SurfaceFormatKHR,
};

pub fn shadow_mapping(device: &Device, shadow_depth_format: vk::Format) -> RenderPass {
    let depth_attachment = AttachmentDescription {
        format: shadow_depth_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        ..Default::default()
    };
    let depth_reference = AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };
    let spd = SubpassDescription {
        pipeline_bind_point: PipelineBindPoint::GRAPHICS,
        color_attachment_count: 0,
        p_depth_stencil_attachment: &depth_reference,
        ..Default::default()
    };
    let dependencies = vec![
        vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL,
            dst_subpass: 0,
            src_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER, // Where it was last used (reading)
            dst_stage_mask: vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS, // Where it will be used now
            src_access_mask: vk::AccessFlags::SHADER_READ,
            dst_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::BY_REGION,
        },
        // 2. Dependency for the END of the pass
        // Transitions from DEPTH_STENCIL_ATTACHMENT_OPTIMAL (Subpass 0) to SHADER_READ_ONLY_OPTIMAL (EXTERNAL)
        vk::SubpassDependency {
            src_subpass: 0,
            dst_subpass: vk::SUBPASS_EXTERNAL,
            src_stage_mask: vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            dst_stage_mask: vk::PipelineStageFlags::FRAGMENT_SHADER,
            src_access_mask: vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            dst_access_mask: vk::AccessFlags::SHADER_READ,
            dependency_flags: vk::DependencyFlags::BY_REGION,
        },
    ];
    let rp_info = RenderPassCreateInfo {
        s_type: StructureType::RENDER_PASS_CREATE_INFO,
        attachment_count: 1,
        p_attachments: &depth_attachment,
        subpass_count: 1,
        p_subpasses: &spd,
        dependency_count: dependencies.len() as u32,
        p_dependencies: dependencies.as_ptr(),
        ..Default::default()
    };
    unsafe { device.create_render_pass(&rp_info, None) }.unwrap()
}

pub fn color_and_depth(format: SurfaceFormatKHR, device: &Device, depth_format: vk::Format) -> RenderPass {
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

    let depth_attachment = AttachmentDescription {
        format: depth_format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };
    let depth_attachment_ref = AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };

    let subpass = SubpassDescription {
        pipeline_bind_point: PipelineBindPoint::GRAPHICS,
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        p_depth_stencil_attachment: &depth_attachment_ref,
        ..Default::default()
    };
    // describe transition of memory layout: implicit step at the beginning to create initial
    // layout
    let subpass_dependency = SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL, // implicit: src before render pass
        dst_subpass: 0,                    // idx to out only subpass
        // waiting on these
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
        src_access_mask: AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        // operations to wait are color attachment writing
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
            | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        ..Default::default()
    };
    let attachments = [color_attachment, depth_attachment];
    let render_pass_info = RenderPassCreateInfo {
        s_type: StructureType::RENDER_PASS_CREATE_INFO,
        attachment_count: attachments.len() as u32,
        p_attachments: attachments.as_ptr(),
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: 1,
        p_dependencies: &subpass_dependency,
        ..Default::default()
    };

    unsafe { device.create_render_pass(&render_pass_info, None) }.unwrap()
}
