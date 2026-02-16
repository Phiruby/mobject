use ash::vk::{Buffer, DescriptorPool, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, DeviceMemory, Extent2D, Framebuffer, PhysicalDeviceMemoryProperties, RenderPass, ShaderStageFlags};
use ash::Device;
use crate::{MAX_FRAMES_IN_FLIGHT, buffers, shaders};
/// This specifies the kind of pipeline the mobject needs to be rendered
/// Each pipeline has their own required descriptor set layout that needs to be
/// adhered. Each mobject implementing a specific pipeline is responsible
/// to follow this layout.
enum Pipelines {
    Primitive {
        vertex_buffers: [Buffer; MAX_FRAMES_IN_FLIGHT],
        vertex_buffer_memories: [DeviceMemory; MAX_FRAMES_IN_FLIGHT],
        index_buffers: [Buffer; MAX_FRAMES_IN_FLIGHT],
        index_buffer_memories: [Buffer; MAX_FRAMES_IN_FLIGHT],
        uniform_buffers: [Buffer; MAX_FRAMES_IN_FLIGHT],
        uniform_buffer_memories: [Buffer; MAX_FRAMES_IN_FLIGHT],
        render_pass: RenderPass,
        framebuffers: [Framebuffer; MAX_FRAMES_IN_FLIGHT],
        descriptor_set_layout: DescriptorSetLayout,
        descriptor_sets: [DescriptorSet; MAX_FRAMES_IN_FLIGHT],
    },
    Curved
}

impl Pipelines::Primitive {
    pub fn new(
        logical_device: &Device,
        nvertices: usize,
        nindices: usize,
        // TODO: the pipeline should create its own render pass?
        render_pass: RenderPass,
        framebuffers: Vec<Framebuffer>,
        descriptor_pool: DescriptorPool,
        extent: Extent2D,
        physical_device_memory_properties: PhysicalDeviceMemoryProperties
    ) -> Self {
        let (vertex_buffers, vertex_buffer_memories) = buffers::create_vertex_buffers(logical_device, nvertices, physical_device_memory_properties);

        let (index_buffers, index_buffer_memories) = buffers::create_index_buffers(logical_device, nindices, physical_device_memory_properties);

        // TODO: unhardcode 10; use structure size
        let (uniform_buffers, uniform_buffer_memories, ubo_mapped_memories) = buffers::create_uniform_buffers(logical_device, physical_device_memory_properties, 10);

        let descriptor_set_layout = shaders::create_description_set_layout(
            logical_device,
            [
                DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    stage_flags: ShaderStageFlags::VERTEX,
                    ..Default::default()
                }
            ].to_vec()
        );

        let descriptor_sets = shaders::scene_descriptor_sets(logical_device, descriptor_set_layout, descriptor_pool, &uniform_buffers);
    }
}
