use std::ffi::CString;

use ash::Device;
use ash::vk::{
    self, Extent2D, GraphicsPipelineCreateInfo, Offset2D, Pipeline, PipelineCache,
    PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo,
    PipelineViewportStateCreateInfo, PrimitiveTopology, Rect2D, RenderPass, ShaderModule,
    ShaderModuleCreateInfo, ShaderStageFlags, StructureType, Viewport,
};

use crate::shapes::{Shape, Vertex2D};

fn create_shader_module(shader_code: Vec<u8>, logical_device: &Device) -> ShaderModule {
    let byte_code: Vec<u32> = shader_code
        .chunks_exact(4)
        .map(|chunk| {
            let bytes: [u8; 4] = chunk.try_into().unwrap();
            u32::from_ne_bytes(bytes)
        })
        .collect();
    let shader_create_info = ShaderModuleCreateInfo {
        s_type: StructureType::SHADER_MODULE_CREATE_INFO,
        p_code: byte_code.as_ptr(),
        code_size: shader_code.len(), // code size is in bytes, not the 4 bytes
        ..Default::default()
    };
    unsafe { logical_device.create_shader_module(&shader_create_info, None) }.unwrap()
}

fn input_assembly_info<'a>() -> PipelineInputAssemblyStateCreateInfo<'a> {
    PipelineInputAssemblyStateCreateInfo {
        s_type: StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        topology: PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: vk::FALSE,
        ..Default::default()
    }
}

fn rasterization_create_info<'a>() -> PipelineRasterizationStateCreateInfo<'a> {
    PipelineRasterizationStateCreateInfo {
        s_type: StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        depth_clamp_enable: vk::FALSE,
        rasterizer_discard_enable: vk::FALSE,
        polygon_mode: vk::PolygonMode::FILL,
        line_width: 1.0,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::CLOCKWISE,
        depth_bias_enable: vk::FALSE,
        ..Default::default()
    }
}

fn multisampling_create_info<'a>() -> PipelineMultisampleStateCreateInfo<'a> {
    PipelineMultisampleStateCreateInfo {
        s_type: StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        sample_shading_enable: vk::FALSE,
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    }
}

fn pipeline_layout_create_info<'a>() -> PipelineLayoutCreateInfo<'a> {
    PipelineLayoutCreateInfo {
        s_type: StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        ..Default::default()
    }
}

pub fn create_graphics_pipeline(
    device: &Device,
    extent: Extent2D,
    render_pass: RenderPass,
) -> Pipeline {
    let vertex_bytes = std::fs::read("shaders/vert.spv").unwrap();
    let vertex_shader_module = create_shader_module(vertex_bytes, device);

    let fragment_bytes = std::fs::read("shaders/frag.spv").unwrap();
    let fragment_shader_module = create_shader_module(fragment_bytes, device);

    let entrypoint = CString::new("main").unwrap();
    let vertex_stage_info = PipelineShaderStageCreateInfo {
        s_type: StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        stage: ShaderStageFlags::VERTEX,
        module: vertex_shader_module,
        p_name: entrypoint.as_ptr(),
        ..Default::default()
    };

    let fragment_stage_info = PipelineShaderStageCreateInfo {
        s_type: StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        stage: ShaderStageFlags::FRAGMENT,
        module: fragment_shader_module,
        p_name: entrypoint.as_ptr(),
        ..Default::default()
    };

    let shader_stages = vec![vertex_stage_info, fragment_stage_info];

    let vertex_input_binding = Vertex2D::binding_description();
    let vertex_attribute_description = Vertex2D::attribute_descriptions();
    let vertex_input_info = PipelineVertexInputStateCreateInfo {
        s_type: StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        vertex_binding_description_count: 1,
        p_vertex_binding_descriptions: &vertex_input_binding,
        vertex_attribute_description_count: vertex_attribute_description.len() as u32,
        p_vertex_attribute_descriptions: vertex_attribute_description.as_ptr(),
        ..Default::default()
    };
    let input_assembly_info = input_assembly_info();
    let viewport = Viewport {
        x: 0.0,
        y: 0.0,
        width: extent.width as f32,
        height: extent.height as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    };
    let sciccors = Rect2D {
        offset: Offset2D { x: 0, y: 0 },
        extent,
    };

    let viewport_create_info = PipelineViewportStateCreateInfo {
        s_type: StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        viewport_count: 1,
        p_viewports: &viewport,
        scissor_count: 1,
        p_scissors: &sciccors,
        ..Default::default()
    };
    let rasterizatio_info = rasterization_create_info();
    let multisample_info = multisampling_create_info();
    let color_attachment = PipelineColorBlendAttachmentState {
        color_write_mask: vk::ColorComponentFlags::RGBA,
        blend_enable: vk::FALSE,
        ..Default::default()
    };
    let color_blend_info = PipelineColorBlendStateCreateInfo {
        s_type: StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        logic_op_enable: vk::FALSE,
        attachment_count: 1,
        logic_op: vk::LogicOp::COPY,
        p_attachments: &color_attachment,
        blend_constants: [0.0, 0.0, 0.0, 0.0],
        ..Default::default()
    };

    let pipeline_layout_info = pipeline_layout_create_info();
    let pipeline_layout =
        unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();

    let pipeline_crate_info = GraphicsPipelineCreateInfo {
        s_type: StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        stage_count: 2,
        p_stages: shader_stages.as_ptr(),
        p_vertex_input_state: &vertex_input_info,
        p_input_assembly_state: &input_assembly_info,
        p_viewport_state: &viewport_create_info,
        p_rasterization_state: &rasterizatio_info,
        p_multisample_state: &multisample_info,
        p_color_blend_state: &color_blend_info,
        p_depth_stencil_state: std::ptr::null(),
        layout: pipeline_layout,
        render_pass,
        subpass: 0,
        ..Default::default()
    };
    let all_pipelines = vec![pipeline_crate_info];
    // taking 0 index since we've created just one pipeline
    unsafe { device.create_graphics_pipelines(PipelineCache::null(), &all_pipelines, None) }
        .unwrap()[0]
}
