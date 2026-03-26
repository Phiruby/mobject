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
    let pool_sizes = [
        DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: MAX_FRAMES_IN_FLIGHT,
        },
        DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: MAX_FRAMES_IN_FLIGHT * 10
        }
    ];
    let pool_info = DescriptorPoolCreateInfo {
        s_type: StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        pool_size_count: pool_sizes.len() as u32,
        p_pool_sizes: pool_sizes.as_ptr(),
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
