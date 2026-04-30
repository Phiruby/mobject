use std::collections::HashMap;

use ash::vk::{self, CommandBuffer, DescriptorPool, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, Extent2D, Framebuffer, IndexType, PhysicalDeviceMemoryProperties, Pipeline, PipelineBindPoint, PipelineLayout, PrimitiveTopology, PushConstantRange, RenderPass, ShaderStageFlags, StructureType, SurfaceFormatKHR, VertexInputAttributeDescription, VertexInputBindingDescription, WriteDescriptorSet};
use ash::Device;
use crate::scene::{Mobject, Texture};
use crate::shapes::{BuiltShape, UBO, Vertex, RenderVertex};
use crate::{MAX_FRAMES_IN_FLIGHT, buffers, render_pass, shaders, window};
use crate::pipelines::{self, CompletePipeline, GraphicsPipeline, PipelineState};

pub struct BezierPipeline {
    // device: &'a Device,
    state: PipelineState
}


impl GraphicsPipeline for BezierPipeline {

    fn get_pipeline_info(device: &Device, extent: Extent2D, depth_format: vk::Format, surface_format: SurfaceFormatKHR) -> CompletePipeline<'static> {
        CompletePipeline {
            extent,
            vertex_path: "shaders/bezier/vert.spv",
            fragment_path: "shaders/bezier/frag.spv",
            tesc_path: Some("shaders/bezier/tes_ctrl.spv"),
            tese_path: Some("shaders/bezier/tes_eval.spv"),
            vertex_binding_description: Self::vertex_binding_description(),
            vertex_attribute_description: Self::vertex_attribute_description().into(),
            topology: PrimitiveTopology::PATCH_LIST,
            color_attachment_count: 1,
            render_pass: Self::render_pass(device, extent, depth_format, surface_format),
            // use to index texture element
            push_constant: Some(
                PushConstantRange {
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    offset: 0,
                    size: 4
                }
            ),
            rasterization_info: shaders::rasterization_create_info()
        }
    }

    fn render_pass(device: &Device, _: Extent2D, depth_format: vk::Format, surface_format: SurfaceFormatKHR) -> RenderPass {
        render_pass::color_and_depth(surface_format, device, depth_format)
    }

    fn mobject_descriptor_set_layout(device: &Device) -> DescriptorSetLayout {
        shaders::create_description_set_layout(
            device,
            [
                DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    stage_flags: ShaderStageFlags::VERTEX | ShaderStageFlags::TESSELLATION_EVALUATION,
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

impl BezierPipeline {
    pub fn new(
        logical_device: &Device,
        swapchain_image_views: &[vk::ImageView],
        depth_image_view: vk::ImageView,
        nvertices: usize,
        nindices: usize,
        scene_descriptor_layout: DescriptorSetLayout,
        extent: Extent2D,
        surface_format: SurfaceFormatKHR,
        depth_format: vk::Format,
        physical_device_memory_properties: PhysicalDeviceMemoryProperties,
    ) -> Self {
        let render_pass = Self::render_pass(logical_device, extent, depth_format, surface_format);
        let attachments = swapchain_image_views.iter().map(|&image_view| [image_view, depth_image_view]).collect();
        let framebuffers = buffers::create_frame_buffers(logical_device, render_pass, attachments, extent);
        Self {
            state: PipelineState::new::<BezierPipeline>(
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
        texture_indices: &HashMap<String, Texture>,
    ) {
        self.state.draw_frame(device, cmd_buffer, frame_index, scene_descriptor_set, mobjects, mobject_descriptor_sets, texture_indices);
    }

    pub fn fill_buffers(&self, frame_index: usize, device: &Device, vertices: &[RenderVertex], indices: &[u32], mobjects: &[&Mobject]) {
        self.state.fill_buffers(frame_index, device, vertices, indices, mobjects);
    }
}
