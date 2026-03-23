use std::ffi::CString;
use std::os::raw::c_void;

use ash::Device;
use ash::vk::{
    self, Buffer, DescriptorBindingFlags, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPool, DescriptorPoolCreateFlags, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutBindingFlagsCreateInfo, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, Extent2D, GraphicsPipelineCreateInfo, ImageView, Offset2D, Pipeline, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineDepthStencilStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo, PipelineTessellationStateCreateFlags, PipelineTessellationStateCreateInfo, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PrimitiveTopology, Rect2D, RenderPass, Sampler, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags, StructureType, Viewport, WriteDescriptorSet
};

use crate::MAX_FRAMES_IN_FLIGHT;
use crate::shapes::{Shape, Vertex, Vertex2D};

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

pub fn rasterization_create_info<'a>() -> PipelineRasterizationStateCreateInfo<'a> {
    PipelineRasterizationStateCreateInfo {
        s_type: StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        depth_clamp_enable: vk::FALSE,
        rasterizer_discard_enable: vk::FALSE,
        polygon_mode: vk::PolygonMode::FILL,
        line_width: 1.0,
        cull_mode: vk::CullModeFlags::NONE,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        depth_bias_enable: vk::FALSE,
        ..Default::default()
    }
}

pub fn multisampling_create_info<'a>() -> PipelineMultisampleStateCreateInfo<'a> {
    PipelineMultisampleStateCreateInfo {
        s_type: StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        sample_shading_enable: vk::FALSE,
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    }
}

pub fn create_description_set_layout(
    device: &Device,
    bindings: Vec<DescriptorSetLayoutBinding>,
    binding_flags: Option<Vec<DescriptorBindingFlags>>,
    create_flags: Option<DescriptorSetLayoutCreateFlags>,
) -> DescriptorSetLayout {
    let flags = binding_flags.unwrap_or_else(Vec::new);
    let create_flags = create_flags.unwrap_or(DescriptorSetLayoutCreateFlags::empty());
    let binding_flags_info = DescriptorSetLayoutBindingFlagsCreateInfo {
        s_type: StructureType::DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO,
        p_binding_flags: flags.as_ptr(),
        binding_count: flags.len() as u32,
        ..Default::default()
    };
    let layout_info = DescriptorSetLayoutCreateInfo {
        s_type: StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        binding_count: bindings.len() as u32,
        p_bindings: bindings.as_ptr(),
        p_next: (&binding_flags_info as *const DescriptorSetLayoutBindingFlagsCreateInfo) as *const c_void,
        flags: create_flags,
        ..Default::default()
    };
    unsafe { device.create_descriptor_set_layout(&layout_info, None) }.unwrap()
}

pub fn scene_descriptor_pool(
    device: &Device,
    flags: Option<DescriptorPoolCreateFlags>
) -> DescriptorPool {
    let flags = flags.unwrap_or(DescriptorPoolCreateFlags::empty());
    let pool_sizes = DescriptorPoolSize {
        ty: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: MAX_FRAMES_IN_FLIGHT,
    };
    let pool_info = DescriptorPoolCreateInfo {
        s_type: StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        pool_size_count: 1,
        p_pool_sizes: &pool_sizes,
        max_sets: MAX_FRAMES_IN_FLIGHT,
        flags,
        ..Default::default()
    };
    unsafe { device.create_descriptor_pool(&pool_info, None) }.unwrap()
}

pub fn scene_descriptor_sets(
    device: &Device,
    scene_descriptor_set_layout: DescriptorSetLayout,
    scene_descriptor_pool: DescriptorPool,
    uniform_buffers: &[Buffer; MAX_FRAMES_IN_FLIGHT as usize],
) -> [DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize] {
    let layouts = [scene_descriptor_set_layout; MAX_FRAMES_IN_FLIGHT as usize];
    let alloc_info = DescriptorSetAllocateInfo {
        s_type: StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        descriptor_pool: scene_descriptor_pool,
        descriptor_set_count: MAX_FRAMES_IN_FLIGHT,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };
    let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info) }.unwrap();

    (0..MAX_FRAMES_IN_FLIGHT).for_each(|i| {
        let buffer_info = DescriptorBufferInfo {
            buffer: uniform_buffers[i as usize],
            offset: 0,
            range: vk::WHOLE_SIZE,
        };
        let descriptor_writes = [WriteDescriptorSet {
            s_type: StructureType::WRITE_DESCRIPTOR_SET,
            dst_set: descriptor_sets[i as usize],
            dst_binding: 0,
            dst_array_element: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            p_buffer_info: &buffer_info,
            ..Default::default()
        }];
        unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };
    });
    descriptor_sets.try_into().unwrap()
}

pub fn mobject_descriptor_sets(
    device: &Device,
    descriptor_set_layout: DescriptorSetLayout,
    descriptor_pool: DescriptorPool,
    uniform_buffers: &[Buffer; MAX_FRAMES_IN_FLIGHT as usize],
    texture_image_view: ImageView,
    sampler: Sampler,
) -> [DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize] {
    let layouts = [descriptor_set_layout; MAX_FRAMES_IN_FLIGHT as usize];
    let alloc_info = DescriptorSetAllocateInfo {
        s_type: StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        descriptor_pool,
        descriptor_set_count: MAX_FRAMES_IN_FLIGHT,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };
    let descriptor_setes = unsafe { device.allocate_descriptor_sets(&alloc_info) }.unwrap();

    (0..MAX_FRAMES_IN_FLIGHT).for_each(|i| {
        let buffer_info = DescriptorBufferInfo {
            buffer: uniform_buffers[i as usize],
            offset: 0,
            range: vk::WHOLE_SIZE,
        };
        let descriptor_writes = [
            WriteDescriptorSet {
                s_type: StructureType::WRITE_DESCRIPTOR_SET,
                dst_set: descriptor_setes[i as usize],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &buffer_info,
                ..Default::default()
            },
        ];
        unsafe { device.update_descriptor_sets(&descriptor_writes, &[]) };
    });
    descriptor_setes.try_into().unwrap()
}

pub fn create_graphics_pipeline(
    device: &Device,
    extent: Extent2D,
    render_pass: RenderPass,
    scene_descriptor_set_layout: DescriptorSetLayout,
    mobject_descriptor_set_layout: DescriptorSetLayout,
) -> (Pipeline, PipelineLayout) {
    let vertex_bytes = std::fs::read("shaders/vert.spv").unwrap();
    let vertex_shader_module = create_shader_module(vertex_bytes, device);

    let fragment_bytes = std::fs::read("shaders/frag.spv").unwrap();
    let fragment_shader_module = create_shader_module(fragment_bytes, device);

    let tesc_bytes = std::fs::read("shaders/tes_ctrl.spv").unwrap();
    let tesc_shader_module = create_shader_module(tesc_bytes, device);

    let tese_bytes = std::fs::read("shaders/tes_eval.spv").unwrap();
    let tese_shader_module = create_shader_module(tese_bytes, device);

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

    let tesc_info = PipelineShaderStageCreateInfo {
        s_type: StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        stage: ShaderStageFlags::TESSELLATION_CONTROL,
        module: tesc_shader_module,
        p_name: entrypoint.as_ptr(),
        ..Default::default()
    };

    let tese_info = PipelineShaderStageCreateInfo {
        s_type: StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        stage: ShaderStageFlags::TESSELLATION_EVALUATION,
        module: tese_shader_module,
        p_name: entrypoint.as_ptr(),
        ..Default::default()
    };

    let shader_stages = vec![vertex_stage_info, tesc_info, tese_info, fragment_stage_info];

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
    let input_assembly_info = PipelineInputAssemblyStateCreateInfo {
        s_type: StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        topology: PrimitiveTopology::PATCH_LIST,
        primitive_restart_enable: vk::FALSE,
        ..Default::default()
    };
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

    let descriptor_set_layouts = [scene_descriptor_set_layout, mobject_descriptor_set_layout];
    let pipeline_layout_info = PipelineLayoutCreateInfo {
        s_type: StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        set_layout_count: descriptor_set_layouts.len() as u32,
        p_set_layouts: descriptor_set_layouts.as_ptr(),
        ..Default::default()
    };
    let pipeline_layout =
        unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();

    let tesselation_info = PipelineTessellationStateCreateInfo {
        s_type: StructureType::PIPELINE_TESSELLATION_STATE_CREATE_INFO,
        flags: PipelineTessellationStateCreateFlags::empty(),
        patch_control_points: 9,
        ..Default::default()
    };
    let depth_stencil_info = PipelineDepthStencilStateCreateInfo {
        s_type: StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        depth_test_enable: vk::TRUE,
        depth_write_enable: vk::TRUE,
        depth_compare_op: vk::CompareOp::LESS,
        depth_bounds_test_enable: vk::FALSE,
        min_depth_bounds: 0.0,
        max_depth_bounds: 1.0,
        stencil_test_enable: vk::FALSE,
        ..Default::default()
    };
    let pipeline_crate_info = GraphicsPipelineCreateInfo {
        s_type: StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        stage_count: shader_stages.len() as u32,
        p_stages: shader_stages.as_ptr(),
        p_vertex_input_state: &vertex_input_info,
        p_input_assembly_state: &input_assembly_info,
        p_viewport_state: &viewport_create_info,
        p_rasterization_state: &rasterizatio_info,
        p_multisample_state: &multisample_info,
        p_color_blend_state: &color_blend_info,
        p_depth_stencil_state: &depth_stencil_info,
        p_tessellation_state: &tesselation_info,
        layout: pipeline_layout,
        render_pass,
        subpass: 0,
        ..Default::default()
    };
    let all_pipelines = vec![pipeline_crate_info];
    // taking 0 index since we've created just one pipeline
    (
        unsafe { device.create_graphics_pipelines(PipelineCache::null(), &all_pipelines, None) }
            .unwrap()[0],
        pipeline_layout,
    )
}
