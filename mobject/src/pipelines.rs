pub mod primitive;
pub mod bezier;
pub mod shadow;
use std::collections::HashMap;
use std::ffi::CString;
pub use primitive::PrimitivePipeline;
pub use bezier::BezierPipeline;
use ash::vk::{self, CommandBuffer, DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetLayout, Extent2D, GraphicsPipelineCreateInfo, Offset2D, Pipeline, PipelineBindPoint, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineDepthStencilStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo, PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo, PipelineTessellationStateCreateFlags, PipelineTessellationStateCreateInfo, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PrimitiveTopology, PushConstantRange, Rect2D, RenderPass, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags, StructureType, VertexInputAttributeDescription, VertexInputBindingDescription, Viewport};
use ash::Device;
use crate::scene::Texture;
use crate::shapes::{BuiltShape};
use crate::{MAX_FRAMES_IN_FLIGHT, shaders};
/// This specifies the kind of pipeline the mobject needs to be rendered
/// Each pipeline has their own required descriptor set layout that needs to be
/// adhered. Each mobject implementing a specific pipeline is responsible
/// to follow this layout.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pipelines {
    Primitive,
    Bezier
}

struct CompletePipeline<'a> {
    extent: Extent2D,
    vertex_path: &'a str,
    fragment_path: &'a str,
    tesc_path: Option<&'a str>,
    tese_path: Option<&'a str>,
    vertex_binding_description: VertexInputBindingDescription,
    vertex_attribute_description: Vec<VertexInputAttributeDescription>,
    topology: PrimitiveTopology,
    render_pass: RenderPass,
    color_attachment_count: u32,
    rasterization_info: PipelineRasterizationStateCreateInfo<'a>,
    push_constant: Option<PushConstantRange>
}

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

fn bind_mobject_shadow(
    device: &Device,
    cmd_buffer: CommandBuffer,
    pipeline_layout: PipelineLayout,
    mobjects: &[&Box<dyn BuiltShape>],
    mobject_descriptor_sets: &[DescriptorSet]
) {
    let mut cummulative_indices = 0;
    for (&descriptor_set, mobj) in mobject_descriptor_sets.iter().zip(mobjects.iter()) {
        unsafe {
            device.cmd_bind_descriptor_sets(cmd_buffer, PipelineBindPoint::GRAPHICS, pipeline_layout, 1, &[descriptor_set], &[])
        };
        let nindices = mobj.indices().len() as u32;
        unsafe {
            device.cmd_draw_indexed(cmd_buffer, nindices, 1, cummulative_indices, 0, 0)
        };
        cummulative_indices += nindices;
    }
}

fn bind_mobject_descriptor_sets(
    device: &Device,
    cmd_buffer: CommandBuffer,
    pipeline_layout: PipelineLayout,
    mobjects: &[&Box<dyn BuiltShape>],
    mobject_descriptor_sets: &[DescriptorSet],
    texture_indices: &HashMap<String, Texture>
) {
    assert_eq!(mobjects.len(), mobject_descriptor_sets.len());
    let mut cummulative_indices = 0;
    mobject_descriptor_sets
    .iter()
    .zip(mobjects)
    .for_each(|(&descriptor_set, mobj)| {
        let pt = String::from(mobj.texture_path().unwrap_or("blank"));
        let texture_index = texture_indices.get(&pt).unwrap().idx as u32;
        let texture_index = texture_index.to_ne_bytes();
        unsafe {
            device.cmd_push_constants(cmd_buffer, pipeline_layout, ShaderStageFlags::FRAGMENT, 0, &texture_index);
        }
        unsafe {
            device.cmd_bind_descriptor_sets(cmd_buffer, PipelineBindPoint::GRAPHICS, pipeline_layout, 1, &[descriptor_set], &[])
        };
        let nindices = mobj.indices().len() as u32;
        unsafe {
            device.cmd_draw_indexed(cmd_buffer, nindices, 1, cummulative_indices, 0, 0)
        };
        cummulative_indices += nindices;
    });
}

fn mobject_descriptor_pool(device: &Device, n_objects: u32, pools: &[DescriptorPoolSize]) -> DescriptorPool {
    let pool_info = DescriptorPoolCreateInfo {
        s_type: StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        pool_size_count: pools.len() as u32,
        p_pool_sizes: pools.as_ptr(),
        max_sets: n_objects * MAX_FRAMES_IN_FLIGHT,
        ..Default::default()
    };
    unsafe {
        device.create_descriptor_pool(&pool_info, None)
        .unwrap()
    }
}


fn create_graphics_pipeline(
    device: &Device,
    extent: Extent2D,
    pipeline_info: CompletePipeline,
    render_pass: RenderPass,
    mobject_descriptor_set_layout: DescriptorSetLayout,
    scene_descriptor_set_layout: DescriptorSetLayout
) -> (Pipeline, PipelineLayout) {
    let vertex_bytes = std::fs::read(pipeline_info.vertex_path).unwrap();
    let vertex_shader_module = create_shader_module(vertex_bytes, device);

    let fragment_bytes = std::fs::read(pipeline_info.fragment_path).unwrap();
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

    let mut shader_stages = vec![vertex_stage_info, fragment_stage_info];

    // if tesc included, assuming tese also included
    if let (Some(tesc_path), Some(tese_path)) = (pipeline_info.tesc_path, pipeline_info.tese_path) {
        let tesc_bytes = std::fs::read(tesc_path).unwrap();
        let tesc_shader_module = create_shader_module(tesc_bytes, device);

        let tese_bytes = std::fs::read(tese_path).unwrap();
        let tese_shader_module = create_shader_module(tese_bytes, device);

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
        shader_stages = vec![
            vertex_stage_info, tesc_info, tese_info, fragment_stage_info
        ];
    }

    let vertex_input_info = PipelineVertexInputStateCreateInfo {
        s_type: StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        vertex_binding_description_count: 1,
        p_vertex_binding_descriptions: &pipeline_info.vertex_binding_description,
        vertex_attribute_description_count: pipeline_info.vertex_attribute_description.len() as u32,
        p_vertex_attribute_descriptions: pipeline_info.vertex_attribute_description.as_ptr(),
        ..Default::default()
    };

    let input_assembly_info = PipelineInputAssemblyStateCreateInfo {
        s_type: StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        topology: pipeline_info.topology,
        primitive_restart_enable: vk::FALSE,
        ..Default::default()
    };
    let viewport = Viewport {
        x: 0.0,
        y: 0.0,
        width: pipeline_info.extent.width as f32,
        height: pipeline_info.extent.height as f32,
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
    let rasterizatio_info = pipeline_info.rasterization_info;
    let multisample_info = shaders::multisampling_create_info();
    let color_attachment = PipelineColorBlendAttachmentState {
        color_write_mask: vk::ColorComponentFlags::RGBA,
        blend_enable: vk::FALSE,
        ..Default::default()
    };
    let color_blend_info = PipelineColorBlendStateCreateInfo {
        s_type: StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        logic_op_enable: vk::FALSE,
        attachment_count: pipeline_info.color_attachment_count,
        logic_op: vk::LogicOp::COPY,
        p_attachments: &color_attachment,
        blend_constants: [0.0, 0.0, 0.0, 0.0],
        ..Default::default()
    };

    let descriptor_set_layouts = [
        scene_descriptor_set_layout,
        mobject_descriptor_set_layout
    ];
    let mut pipeline_layout_info = PipelineLayoutCreateInfo {
        s_type: StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        set_layout_count: descriptor_set_layouts.len() as u32,
        p_set_layouts: descriptor_set_layouts.as_ptr(),
        ..Default::default()
    };
    if let Some(pcr) = pipeline_info.push_constant {
        pipeline_layout_info.push_constant_range_count = 1;
        pipeline_layout_info.p_push_constant_ranges = &pcr;
    }
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
