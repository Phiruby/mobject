use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::offset_of;

use ash::vk::{self, Buffer, ClearColorValue, ClearDepthStencilValue, ClearValue, CommandBuffer, CommandBufferBeginInfo, DescriptorBufferInfo, DescriptorPool, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, DeviceMemory, Extent2D, Framebuffer, IndexType, Offset2D, PhysicalDeviceMemoryProperties, Pipeline, PipelineBindPoint, PipelineLayout, PrimitiveTopology, PushConstantRange, Rect2D, RenderPass, RenderPassBeginInfo, ShaderStageFlags, StructureType, SubpassContents, VertexInputAttributeDescription, VertexInputBindingDescription, WriteDescriptorSet};
use ash::Device;
use crate::scene::{Mobject, Texture};
use crate::shapes::{BuiltShape, UBO, Vertex, Vertex2D};
use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shaders, window};
use crate::pipelines::{self, CompletePipeline};
pub struct PrimitivePipeline {
    // device: &'a Device,
    extent: Extent2D,
    pub vertex_buffers: [Buffer; MAX_FRAMES_IN_FLIGHT as usize],
    pub vertex_buffer_memories: [DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize],
    pub index_buffers: [Buffer; MAX_FRAMES_IN_FLIGHT as usize],
    pub index_buffer_memories: [DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize],
    uniform_buffers: [Buffer; MAX_FRAMES_IN_FLIGHT as usize],
    uniform_buffer_memories: [DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize],
    uniform_buffer_mapped_memories: [*mut c_void; MAX_FRAMES_IN_FLIGHT as usize],
    render_pass: RenderPass,
    framebuffers: [Framebuffer; MAX_FRAMES_IN_FLIGHT as usize],
    mobject_descriptor_set_layout: DescriptorSetLayout,
    mobject_descriptor_pool: DescriptorPool,
    pipeline: Pipeline,
    pipeline_layout: PipelineLayout
}

impl PrimitivePipeline {
    pub fn new(
        logical_device: &Device,
        nvertices: usize,
        nindices: usize,
        // TODO: the pipeline should create its own render pass?
        render_pass: RenderPass,
        framebuffers: [Framebuffer; 3],
        scene_descriptor_layout: DescriptorSetLayout,
        extent: Extent2D,
        physical_device_memory_properties: PhysicalDeviceMemoryProperties,
    ) -> Self {
        let (vertex_buffers, vertex_buffer_memories) = buffers::create_vertex_buffers(logical_device, nvertices, physical_device_memory_properties);

        let (index_buffers, index_buffer_memories) = buffers::create_index_buffers(logical_device, nindices, physical_device_memory_properties);

        // TODO: unhardcode 10; use structure size
        let (uniform_buffers, uniform_buffer_memories, ubo_mapped_memories) = buffers::create_uniform_buffers(logical_device, physical_device_memory_properties, std::mem::size_of::<UBO>() as u64);
        let mobject_descriptor_set_layout = shaders::create_description_set_layout(
            logical_device,
            [
                DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    stage_flags: ShaderStageFlags::VERTEX,
                    ..Default::default()
                }
            ].to_vec(),
            None,
            None
        );
        let mobject_descriptor_pool = pipelines::mobject_descriptor_pool(
            logical_device,
            10,
            &[
                DescriptorPoolSize {
                    ty: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 10 * MAX_FRAMES_IN_FLIGHT
                }
            ]
        );

        let pipeline_info = CompletePipeline {
            extent,
            vertex_path: "shaders/vert.spv",
            fragment_path: "shaders/frag.spv",
            tesc_path: None,
            tese_path: None,
            vertex_binding_description: Self::vertex_binding_description(),
            vertex_attribute_description: Self::vertex_attribute_description().into(),
            topology: PrimitiveTopology::TRIANGLE_LIST,
            render_pass,
            // use to index texture element
            push_constant: Some(
                PushConstantRange {
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    offset: 0,
                    size: 4
                }
            )
        };
        let (pipeline, layout) = pipelines::create_graphics_pipeline(
            logical_device,
            extent,
            pipeline_info,
            render_pass,
            mobject_descriptor_set_layout,
            scene_descriptor_layout
        );
        Self {
            extent,
            vertex_buffers,
            vertex_buffer_memories,
            index_buffers,
            index_buffer_memories,
            uniform_buffers,
            uniform_buffer_memories,
            uniform_buffer_mapped_memories: ubo_mapped_memories,
            render_pass,
            framebuffers,
            mobject_descriptor_set_layout,
            mobject_descriptor_pool,
            pipeline,
            pipeline_layout: layout
        }
    }

    pub fn create_mobject_descriptor_sets(
        &self,
        device: &Device,
        uniform_buffers: &[Buffer; MAX_FRAMES_IN_FLIGHT as usize],
    ) -> [DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize] {
        let layouts = [
            self.mobject_descriptor_set_layout;
            MAX_FRAMES_IN_FLIGHT as usize
        ];
        let alloc_info = DescriptorSetAllocateInfo {
            s_type: StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
            descriptor_pool: self.mobject_descriptor_pool,
            descriptor_set_count: MAX_FRAMES_IN_FLIGHT,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };
        let descriptor_sets = unsafe {
            device.allocate_descriptor_sets(&alloc_info)
            .unwrap()
        };
        (0..MAX_FRAMES_IN_FLIGHT)
            .for_each(|i| {
                let uniform_buffer_info = DescriptorBufferInfo {
                    buffer: uniform_buffers[i as usize],
                    offset: 0,
                    range: vk::WHOLE_SIZE
                };
                let write_op = [
                    WriteDescriptorSet {
                        s_type: StructureType::WRITE_DESCRIPTOR_SET,
                        dst_set: descriptor_sets[i as usize],
                        dst_binding: 0,
                        dst_array_element: 0,
                        descriptor_type: DescriptorType::UNIFORM_BUFFER,
                        descriptor_count: 1,
                        p_buffer_info: &uniform_buffer_info,
                        ..Default::default()
                    }
                ];
                unsafe {
                    device.update_descriptor_sets(&write_op, &[]);
                }
            });
        descriptor_sets.try_into().unwrap()

    }
    pub fn draw_frame(
        &self,
        device: &Device,
        cmd_buffer: CommandBuffer,
        frame_index: usize,
        scene_descriptor_set: DescriptorSet,
        // TODO: annotate with primitive trait as well
        mobjects: &[&Box<dyn BuiltShape>],
        mobject_descriptor_sets: &[DescriptorSet],
        texture_indices: &HashMap<String, Texture>
    ) {

        unsafe {
            device.cmd_bind_pipeline(cmd_buffer, PipelineBindPoint::GRAPHICS, self.pipeline)
        };

        let vertex_buffer = self.vertex_buffers[frame_index];
        let index_buffer = self.index_buffers[frame_index];
        unsafe {
            device.cmd_bind_vertex_buffers(cmd_buffer, 0, &[vertex_buffer], &[0])
        };
        unsafe {
            device.cmd_bind_index_buffer(cmd_buffer, index_buffer, 0, IndexType::UINT32)
        };
        // bind scene-level info
        unsafe {
            device.cmd_bind_descriptor_sets(cmd_buffer, PipelineBindPoint::GRAPHICS, self.pipeline_layout, 0, &[scene_descriptor_set], &[])
        };

        pipelines::bind_mobject_descriptor_sets(device, cmd_buffer, self.pipeline_layout, mobjects, mobject_descriptor_sets, texture_indices);
    }

    /// Fills the index, vertex, and uniform buffers
    pub fn fill_buffers(&self, frame_index: usize, device: &Device, vertices: &[Vertex2D], indices: &[u32], mobjects: &[&Mobject]) {
        window::fill_vertex_buffer(device, self.vertex_buffer_memories[frame_index], vertices);
        window::fill_index_buffer(device, self.index_buffer_memories[frame_index], indices);
        mobjects
            .iter()
            .for_each(|mobj| {
                let (_, _, mapped) = mobj.get_uniform_buffer();
                window::fill_uniform_buffer(
                    mapped[frame_index],
                    mobj.get_ubo_contents()
                );
            });
    }

    fn vertex_binding_description() -> VertexInputBindingDescription {
        VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<Vertex2D>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    fn vertex_attribute_description() -> [VertexInputAttributeDescription; 4] {
        Vertex2D::attribute_descriptions()
    }
}
