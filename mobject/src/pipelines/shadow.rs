use std::collections::HashMap;
use std::ffi::c_void;
use ash::Instance;
use ash::vk::{self, Buffer, ClearDepthStencilValue, ClearValue, CommandBuffer, CommandBufferBeginInfo, DescriptorBufferInfo, DescriptorImageInfo, DescriptorPool, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, DeviceMemory, Extent2D, Format, Framebuffer, Image, ImageLayout, ImageUsageFlags, ImageView, IndexType, Offset2D, PhysicalDevice, PhysicalDeviceMemoryProperties, Pipeline, PipelineBindPoint, PipelineLayout, PrimitiveTopology, PushConstantRange, Rect2D, RenderPass, RenderPassBeginInfo, ShaderStageFlags, StructureType, SubpassContents, SurfaceFormatKHR, VertexInputAttributeDescription, VertexInputBindingDescription, WriteDescriptorSet};
use ash::Device;
use crate::scene::{Mobject, Texture};
use crate::shapes::{BuiltShape, UBO, Vertex, RenderVertex};
use crate::{MAX_FRAMES_IN_FLIGHT, buffers, render_pass, shaders, texture, window};
use crate::pipelines::{self, CompletePipeline, GraphicsPipeline, PipelineState};

/// This pipeline is used for shadow mapping:
/// its main goal is to produce a depth map from the point of view
/// of the light sources
pub struct ShadowMapping {
    state: PipelineState,
}

impl GraphicsPipeline for ShadowMapping {
    fn get_pipeline_info(render_pass: RenderPass, extent: Extent2D) -> CompletePipeline<'static> {
        CompletePipeline {
            extent,
            vertex_path: "shaders/shadow/vert.spv",
            fragment_path: "shaders/shadow/frag.spv",
            tesc_path: None,
            tese_path: None,
            vertex_binding_description: Self::vertex_binding_description(),
            vertex_attribute_description: Self::vertex_attribute_description().into(),
            topology: PrimitiveTopology::TRIANGLE_LIST,
            color_attachment_count: 0,
            render_pass,
            // use to index texture element
            push_constant: Some(
                PushConstantRange {
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    offset: 0,
                    size: 4
                }
            ),
            rasterization_info: shaders::shadow_rasterization_create_info()
        }
    }

    fn render_pass(device: &Device, _: Extent2D, depth_format: vk::Format, _: SurfaceFormatKHR) -> RenderPass {
        render_pass::shadow_mapping(device, depth_format)
    }

    fn mobject_descriptor_set_layout(device: &Device) -> DescriptorSetLayout {
        shaders::create_description_set_layout(
            device,
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
        )
    }
    fn mobject_descriptor_pool(device: &Device) -> DescriptorPool {
        pipelines::mobject_descriptor_pool(
            device,
            10,
            &[
                DescriptorPoolSize {
                    ty: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 10 * MAX_FRAMES_IN_FLIGHT
                }
            ]
        )
    }

    fn vertex_binding_description() -> VertexInputBindingDescription {
        VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<RenderVertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    fn vertex_attribute_description() -> Vec<VertexInputAttributeDescription> {
        RenderVertex::attribute_descriptions().try_into().unwrap()
    }
}

impl ShadowMapping {
    pub fn new(
        logical_device: &Device,
        physical_device: PhysicalDevice,
        instance: &Instance,
        nvertices: usize,
        nindices: usize,
        scene_descriptor_layout: DescriptorSetLayout,
        scene_descriptor_sets: &[DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize],
        extent: Extent2D,
        surface_format: SurfaceFormatKHR,
        physical_device_memory_properties: PhysicalDeviceMemoryProperties,
    ) -> Self {
        let shadowmaps: Vec<(Image, ImageView, DeviceMemory, Format)> = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| {
                buffers::create_depth_buffer(
                    &instance,
                    &logical_device,
                    physical_device,
                    extent,
                    ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | ImageUsageFlags::SAMPLED,
                    physical_device_memory_properties,
                )
            })
            .collect();

        let shadow_depth_image_views: Vec<ImageView> = shadowmaps.iter().map(|(_, image_view, _, _)| *image_view).collect();

        let depth_format = shadowmaps[0].3;

        let render_pass = Self::render_pass(logical_device, extent, depth_format, surface_format);
        let attachments = shadow_depth_image_views.iter().map(|&image_view| [image_view]).collect();
        let framebuffers = buffers::create_frame_buffers(logical_device, render_pass, attachments, extent);
        let shadow_sampler = texture::create_sampler(&logical_device, &instance, physical_device, vk::TRUE, vk::CompareOp::LESS);
        scene_descriptor_sets
            .iter()
            .enumerate()
            .for_each(|(i, desc_set)| {
                let image_info = DescriptorImageInfo {
                    image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    image_view: shadow_depth_image_views[i],
                    sampler: shadow_sampler,
                    ..Default::default()
                };

                let write = WriteDescriptorSet {
                    s_type: StructureType::WRITE_DESCRIPTOR_SET,
                    dst_set: *desc_set,
                    dst_binding: 2,
                    dst_array_element: 0,
                    descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    p_image_info: &image_info,
                    ..Default::default()
                };

                unsafe { logical_device.update_descriptor_sets(&[write], &[]) };
            });
        Self {
            state: PipelineState::new::<ShadowMapping>(
                logical_device,
                nvertices,
                nindices,
                scene_descriptor_layout,
                render_pass,
                framebuffers.try_into().unwrap(),
                extent,
                surface_format,
                depth_format,
                physical_device_memory_properties
            )
        }
    }
    pub fn draw_frame(
        &self,
        device: &Device,
        cmd_buffer: CommandBuffer,
        frame_index: usize,
        scene_descriptor_set: DescriptorSet,
        mobjects: &[&Box<dyn BuiltShape>],
        mobject_descriptor_sets: &[DescriptorSet],
    ) {
        let shadow_clear_values = [
            ClearValue {
                depth_stencil: ClearDepthStencilValue { depth: 1.0, stencil: 0 }
            }
        ];

        let shadow_pass_begin_info = RenderPassBeginInfo {
            render_pass: self.state.render_pass,
            framebuffer: self.state.framebuffers[frame_index],
            render_area: Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: self.state.extent
            },
            clear_value_count: 1,
            p_clear_values: shadow_clear_values.as_ptr(),
            ..Default::default()
        };
        unsafe {
            device.cmd_begin_render_pass(cmd_buffer, &shadow_pass_begin_info, SubpassContents::INLINE);
        };

        unsafe {
            device.cmd_bind_pipeline(cmd_buffer, PipelineBindPoint::GRAPHICS, self.state.pipeline)
        };

        let vertex_buffer = self.state.vertex_buffers[frame_index];
        let index_buffer = self.state.index_buffers[frame_index];
        unsafe {
            device.cmd_bind_vertex_buffers(cmd_buffer, 0, &[vertex_buffer], &[0])
        };
        unsafe {
            device.cmd_bind_index_buffer(cmd_buffer, index_buffer, 0, IndexType::UINT32)
        };
        // bind scene-level info
        unsafe {
            device.cmd_bind_descriptor_sets(cmd_buffer, PipelineBindPoint::GRAPHICS, self.state.pipeline_layout, 0, &[scene_descriptor_set], &[])
        };
        pipelines::bind_mobject_shadow(device, cmd_buffer, self.state.pipeline_layout, mobjects, mobject_descriptor_sets);

        unsafe { device.cmd_end_render_pass(cmd_buffer) };
    }

    pub fn fill_buffers(&self, frame_index: usize, device: &Device, vertices: &[RenderVertex], indices: &[u32], mobjects: &[&Mobject]) {
        self.state.fill_buffers(frame_index, device, vertices, indices, mobjects);
    }
}
