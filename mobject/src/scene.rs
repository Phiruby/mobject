use ash::vk::{
    self, ApplicationInfo, Buffer, ClearColorValue, ClearDepthStencilValue, ClearValue, CommandBuffer, CommandBufferBeginInfo, CommandBufferResetFlags, CommandPool, DescriptorBindingFlags, DescriptorImageInfo, DescriptorPool, DescriptorPoolCreateFlags, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags, DescriptorType, DeviceMemory, Extent2D, Fence, Framebuffer, Handle, Image, ImageAspectFlags, ImageLayout, ImageView, InstanceCreateInfo, MemoryPropertyFlags, Offset2D, PhysicalDeviceFeatures, PhysicalDeviceMemoryProperties, Pipeline, PipelineLayout, PresentInfoKHR, Queue, Rect2D, RenderPass, RenderPassBeginInfo, Sampler, ShaderStageFlags, StructureType, SubmitInfo, SubpassContents, SurfaceKHR, SwapchainKHR, WriteDescriptorSet
};
use ash::{Device, Entry, Instance, khr, khr::surface};
use image::Frame;
use crate::c_utils::Utf8Pointer;
use crate::device::QueueFamilies;
use crate::pipelines::{BezierPipeline, Pipelines, PrimitivePipeline};
use glfw::PWindow;
use nalgebra_glm as glm;
use crate::shapes::{BuiltShape, GlobalUBO, Shape, UBO};
use std::collections::HashMap;
use std::ffi::{CString, c_void};
use std::ops::Deref;
use crate::{window, swapchain, shaders, render_pass, buffers, texture, shapes, device, pipelines};
use std::time::{Instant, Duration};
use crate::MAX_FRAMES_IN_FLIGHT;
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
const DEVICE_EXTENSIONS: [&str; 2] = ["VK_KHR_swapchain", "VK_EXT_descriptor_indexing"];

enum Action {
    AddMobject(Box<dyn Shape>),
    Wait { seconds: u8 }
}

/// Represents the current state of the scene:
/// Frozen: all movement / actions are blocked. Nothing in the scene can change.
///     Usually in this state when the user explicitly freezes everything
/// Waiting: existing mobjects in the scene are free to animate and move around.
///     But any future action will have to wait (e.g: adding new objects)
/// Moving: all mobjects are freely moving
enum SceneState {
    Frozen,
    Waiting{ from: Instant, duration: Duration},
    Moving
}

pub struct Texture {
    view: ImageView,
    pub idx: u32,
    image: Image,
    memory: DeviceMemory
}

pub struct Mobject {
    mobject: Box<dyn BuiltShape>,
    descriptor_sets: [DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize]
}
impl Deref for Mobject {
    type Target = Box<dyn BuiltShape>;
    fn deref(&self) -> &Self::Target {
        &self.mobject
    }
}

pub struct Scene {
    state: SceneState,
    // mobjects: Vec<Box<dyn BuiltShape>>,
    actions: Vec<Action>,
    sync: Vec<window::Sync>,
    device: Device,
    swapchain_device: khr::swapchain::Device,
    window: PWindow,
    swapchain: SwapchainKHR,
    command_buffer: Vec<CommandBuffer>,
    render_pass: RenderPass,
    uniform_buffer_mapped_memories: Vec<*mut c_void>,
    depth_image: Image,
    depth_image_view: ImageView,
    depth_image_memory: DeviceMemory,
    scene_descriptor_sets: [DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize],
    // mobject_descriptor_sets: Vec<[DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize]>,
    mobjects: Vec<Mobject>,
    extent: Extent2D,
    textures: HashMap<String, Texture>,
    texture_sampler: Sampler,
    queue_families: QueueFamilies,
    global_ubo: GlobalUBO,
    physical_device_properties: PhysicalDeviceMemoryProperties,
    primitive_pipeline: PrimitivePipeline,
    bezier_pipeline: BezierPipeline,
    graphics_queue: Queue,
    cmd_pool: CommandPool,
    framebuffers: Vec<Framebuffer>
}

impl Scene {
    pub fn new() -> Self {
        let entry = unsafe { Entry::load().unwrap() };
        let (window, required_instance_extensions, _window_event_listener) = window::create_glfw_window(700, 700);
        let instance = create_vk_instance(&entry, Some(required_instance_extensions));
        let surface_instance = surface::Instance::new(&entry, &instance);
        let surface = window::create_surface(&instance, &window);
        let ash_surface = SurfaceKHR::from_raw(surface as u64);
        let physical_device = device::select_physical_device(&instance);
        let queue_families =
            QueueFamilies::new(&instance, &surface_instance, ash_surface, physical_device);
        let mut swapchain_capabilities =
            swapchain::query_support(physical_device, &surface_instance, ash_surface);
        let surface_format = swapchain_capabilities.choose_surface_format();
        let present_mode = swapchain_capabilities.choose_present_mode();
        let extent = swapchain_capabilities.choose_extent(&window);
        let logical_device = device::create_logical_device(
            &queue_families,
            &instance,
            physical_device,
            Some(DEVICE_EXTENSIONS.to_vec()),
        );
        let swapchain_device = khr::swapchain::Device::new(&instance, &logical_device);
        let swapchain = swapchain::create(
            &swapchain_device,
            present_mode,
            &swapchain_capabilities.capabilities,
            ash_surface,
            surface_format,
            &queue_families,
            extent,
        );
        let swapchain_images = swapchain::acquire_images(swapchain, &swapchain_device);
        println!("{:?} <-- total swapchain images", swapchain_images.len());
        let swapchain_imageviews = swapchain::create_image_views(
            &logical_device,
            &swapchain_images,
            1,
            surface_format.format,
            ImageAspectFlags::COLOR,
        );

        let pool = buffers::create_command_pool(&logical_device, &queue_families);
        let command_buffer =
            buffers::create_command_buffers(pool, &logical_device, MAX_FRAMES_IN_FLIGHT);
        let sync = window::create_sync_objects(&logical_device);
        let physical_device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let graphics_queue =
            unsafe { logical_device.get_device_queue(0, queue_families.graphics_index as u32) };

        let (uniform_buffers, uniform_buffer_memories, uniform_buffer_mapped_memories) =
            buffers::create_uniform_buffers::<{ MAX_FRAMES_IN_FLIGHT as usize }>(
                &logical_device,
                physical_device_memory_properties,
                size_of::<GlobalUBO>() as u64,
            );
        let (depth_image, depth_image_view, depth_image_memory, depth_image_format) =
            buffers::create_depth_buffer(
                &instance,
                &logical_device,
                physical_device,
                extent,
                physical_device_memory_properties,
            );
        let render_pass = render_pass::create(surface_format, &logical_device, depth_image_format);
        let framebuffers = buffers::create_frame_buffers(
            &logical_device,
            render_pass,
            &swapchain_imageviews,
            depth_image_view,
            extent,
        );
        let texture_sampler = texture::create_sampler(&logical_device, &instance, physical_device);
        let scene_descriptor_set_layout = shaders::create_description_set_layout(
            &logical_device,
            [
                DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    stage_flags: ShaderStageFlags::VERTEX | ShaderStageFlags::TESSELLATION_EVALUATION,
                    ..Default::default()
                },
                DescriptorSetLayoutBinding {
                    binding: 1,
                    descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 10,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    ..Default::default()
                }
            ]
            .to_vec(),
            Some(
                [
                    DescriptorBindingFlags::empty(),
                    DescriptorBindingFlags::PARTIALLY_BOUND | DescriptorBindingFlags::UPDATE_AFTER_BIND
                ]
                .to_vec()
            ),
            Some(
                DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL
            )
        );

        let scene_descriptor_pool = shaders::scene_descriptor_pool(
            &logical_device,
            Some(DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
        );
        let scene_descriptor_sets = shaders::scene_descriptor_sets(
            &logical_device,
            scene_descriptor_set_layout,
            scene_descriptor_pool,
            &uniform_buffers,
        );

        let primitive_pipeline = pipelines::PrimitivePipeline::new(
            &logical_device,
            100000,
            100000,
            render_pass,
            framebuffers.iter().copied().collect::<Vec<Framebuffer>>().try_into().unwrap(),
            scene_descriptor_set_layout,
            extent,
            physical_device_memory_properties
        );

        let bezier_pipeline = pipelines::BezierPipeline::new(
            &logical_device,
            10000,
            10000,
            render_pass,
            framebuffers.iter().copied().collect::<Vec<Framebuffer>>().try_into().unwrap(), scene_descriptor_set_layout,
            extent,
            physical_device_memory_properties
        );

        // let mobject_descriptor_sets: Vec<[DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize]> = Vec::new();

        let camera_position = glm::vec3(2.0, 2.0, 2.0);
        let origin = glm::vec3(0.0, 0.0, 0.0);
        let up = glm::vec3(0.0, 0.0, 1.0);
        let angle = glm::vec1(45.0);
        Self {
            state: SceneState::Moving,
            actions: Vec::new(),
            mobjects: Vec::new(),
            sync,
            device: logical_device,
            swapchain_device,
            window,
            command_buffer,
            swapchain,
            render_pass,
            depth_image,
            depth_image_view,
            depth_image_memory,
            uniform_buffer_mapped_memories: uniform_buffer_mapped_memories.to_vec(),
            scene_descriptor_sets,
            extent,
            textures: HashMap::new(),
            texture_sampler,
            queue_families,
            // TODO: projection can even be moved to a constant ubo
            global_ubo: GlobalUBO {
                camera_position,
                _pad: 0,
                view: glm::look_at(&camera_position, &origin, &up),
                proj: (glm::perspective_zo(
                    (extent.width / extent.height) as f32,
                    glm::radians(&angle).x,
                    0.1,
                    10.0,
                )),
            },
            physical_device_properties: physical_device_memory_properties,
            primitive_pipeline,
            bezier_pipeline,
            graphics_queue,
            cmd_pool: pool,
            framebuffers
        }
    }

    fn add_texture_to_scene(&self, image: Image, memory: DeviceMemory, mip_levels: u32, image_view: ImageView) {
        self.scene_descriptor_sets
            .iter()
            .for_each(|desc_set| {

                let image_info = DescriptorImageInfo {
                    image_layout: ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    image_view: image_view,
                    sampler: self.texture_sampler,
                    ..Default::default()
                };
                let write = WriteDescriptorSet {
                    s_type: StructureType::WRITE_DESCRIPTOR_SET,
                    dst_set: *desc_set,
                    dst_binding: 1,
                    dst_array_element: self.textures.len() as u32,
                    descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    p_image_info: &image_info,
                    ..Default::default()
                };
                unsafe { self.device.update_descriptor_sets(&[write], &[]) };
            });
    }

    pub fn add(&mut self, mobject: Box<dyn Shape>) {
        let texture_path = mobject.texture_path();
        if let Some(pt) = texture_path {
            // TODO: shouldn't return; need to push to actions
            if self.textures.contains_key(pt) { return ;}

            let (image, memory, mip_levels) = texture::create_texture_image(&self.device, pt, self.physical_device_properties, self.cmd_pool, self.graphics_queue);
            let texture_image_view =
            texture::create_texture_image_view(&self.device, image, mip_levels);
            self.add_texture_to_scene(image, memory, mip_levels, texture_image_view);
            let idx = self.textures.len() as u32;
            self.textures.insert(pt.to_string(), Texture { view: texture_image_view, idx, image, memory });
        }
        self.actions.push(Action::AddMobject(mobject));
    }
    pub fn wait(&mut self, seconds: u8) {
        self.actions.push(Action::Wait { seconds });
    }

    fn set_state(&mut self, state: SceneState) {
        self.state = state;
    }

    fn add_ones_texture(&mut self) {
        let (image, memory, mip_levels) = texture::create_ones_texture(&self.device, self.physical_device_properties, self.cmd_pool, self.graphics_queue);
        let image_view = texture::create_texture_image_view(&self.device, image, mip_levels);
        self.add_texture_to_scene(image, memory, mip_levels, image_view);
        let idx = self.textures.len() as u32;
        self.textures.insert(String::from("blank"), Texture { view: image_view, idx, image, memory });
    }

    fn take_action(&mut self) {
        if self.actions.len() == 0 {return ;}
        // NOTE: going backwards. doing this for now for simplicity
        let action = self.actions.pop().unwrap();
        match action {
            Action::AddMobject(mobj) => {
                let build_mobject = mobj.build(&self.device, self.physical_device_properties);
                let mobj_desc_sets = self.make_mobject_descriptor_set(&build_mobject);

                self.mobjects.push(Mobject { mobject: build_mobject, descriptor_sets: mobj_desc_sets });
            },
            Action::Wait { seconds } => self.set_state(SceneState::Waiting{ from: Instant::now(), duration: Duration::from_secs(seconds as u64) }),
        }
    }

    fn make_mobject_descriptor_set(&self, mobject: &Box<dyn BuiltShape>) -> [DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize] {
        let pipeline = mobject.get_pipeline();
        let (uniform_buffers, _, _) = mobject.get_uniform_buffer();
        let built_descriptor_sets = match pipeline {
            Pipelines::Primitive => self.primitive_pipeline.create_mobject_descriptor_sets(&self.device, uniform_buffers),
            Pipelines::Bezier => self.bezier_pipeline.create_mobject_descriptor_sets(&self.device, uniform_buffers)
        };
        built_descriptor_sets
    }

    pub fn main_loop(&mut self) {
        self.add_ones_texture();
        // opengl to vulkan conversion (inverted y)
        self.global_ubo.proj.m22 *= -1.0;
        let graphics_queue = unsafe {
            self.device
                .get_device_queue(0, self.queue_families.graphics_index as u32)
        };
        let present_queue = unsafe {
            self.device
                .get_device_queue(0, self.queue_families.presentation_index as u32)
        };
        let mut current_frame: usize = 0;
        while !(self.window.should_close()) {
            unsafe { glfw::ffi::glfwPollEvents() };
            if let SceneState::Moving = self.state {
                self.take_action();
            }
            self.draw_frame(graphics_queue, present_queue, current_frame);
            current_frame = (current_frame + 1) % (MAX_FRAMES_IN_FLIGHT as usize);
            if let SceneState::Waiting { from, duration } = self.state {
                let elapsed = from.elapsed();
                if elapsed > duration {
                    self.set_state(SceneState::Moving);
                }
            }
        }
    }

    fn group_mobjects(mobjects: &[Mobject]) -> HashMap<Pipelines, Vec<&Mobject>> {
        let mut map: HashMap<Pipelines, Vec<&Mobject>> = HashMap::new();
        mobjects
            .iter()
            .for_each(|mobj| {
                let pipeline = mobj.get_pipeline();
                map.entry(pipeline)
                    .or_insert_with(Vec::new)
                    .push(mobj);
            });
        map
    }

    fn draw_mobjects(&self, current_frame: usize, pipeline: Pipelines, mobjects: &[&Mobject]) {
        let (vertices, indices) = shapes::mobjects_to_vertices_and_indices(mobjects);
        match pipeline {
            Pipelines::Primitive => self.primitive_pipeline.fill_buffers(current_frame, &self.device, &vertices, &indices, mobjects),
            Pipelines::Bezier => self.bezier_pipeline.fill_buffers(current_frame, &self.device, &vertices, &indices, mobjects),
        };
    }

    fn draw_frame(&mut self, graphics_queue: Queue, present_queue: Queue, current_frame: usize) {

        let sync = &self.sync[current_frame];
        let command_buffer = self.command_buffer[current_frame];

        unsafe {
            self.device
                .wait_for_fences(&[sync.in_flight], true, u64::MAX)
        }
        .unwrap();
        unsafe { self.device.reset_fences(&[sync.in_flight]) }.unwrap();
        // TODO: move out of this struct
        let grouped_mobjects = Scene::group_mobjects(&self.mobjects);

        for (&pipeline_kind, mobjs) in grouped_mobjects.iter() {
            self.draw_mobjects(current_frame, pipeline_kind, &mobjs);
        }

        window::fill_uniform_buffer(
            self.uniform_buffer_mapped_memories[current_frame],
            &self.global_ubo,
        );

        let (image_index, _suboptimal) = unsafe {
            self.swapchain_device.acquire_next_image(
                self.swapchain,
                u64::MAX,
                sync.image_available,
                Fence::null(),
            )
        }
        .unwrap();
        unsafe {
            self.device
                .reset_command_buffer(command_buffer, CommandBufferResetFlags::empty())
        }
        .unwrap();

        let cmd_begin_info = CommandBufferBeginInfo {
            s_type: StructureType::COMMAND_BUFFER_BEGIN_INFO,
            ..Default::default()
        };
        unsafe { self.device.begin_command_buffer(command_buffer, &cmd_begin_info) }.unwrap();
        let clear_colors = [
            ClearValue {
                color: ClearColorValue {
                    float32: [0.0, 0.0, 0.0, 0.0]
                },
            },
            ClearValue {
                depth_stencil: ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0
                }
            }
        ];
        let render_pass_begin_info = RenderPassBeginInfo {
            s_type: StructureType::RENDER_PASS_BEGIN_INFO,
            render_pass: self.render_pass,
            framebuffer: self.framebuffers[current_frame],
            render_area: Rect2D { offset: Offset2D { x: 0, y: 0 }, extent: self.extent },
            clear_value_count: clear_colors.len() as u32,
            p_clear_values: clear_colors.as_ptr(),
            ..Default::default()
        };

        unsafe {
            self.device.cmd_begin_render_pass(command_buffer, &render_pass_begin_info, SubpassContents::INLINE)
        };

        for (&pipeline_kind, mobjs) in grouped_mobjects.iter() {
            let mobs: Vec<&Box<dyn BuiltShape>> = mobjs.iter().map(|x| &x.mobject).collect();
            let desc_sets: Vec<DescriptorSet> = mobjs.iter().map(|x| x.descriptor_sets[current_frame]).collect();
            match pipeline_kind {
                Pipelines::Primitive => self.primitive_pipeline.draw_frame(
                    &self.device,
                    command_buffer,
                    current_frame,
                    self.scene_descriptor_sets[current_frame],
                    &mobs,
                    &desc_sets,
                    &self.textures
                ),

                Pipelines::Bezier => self.bezier_pipeline.draw_frame(
                    &self.device,
                    command_buffer,
                    current_frame,
                    self.scene_descriptor_sets[current_frame],
                    &mobs,
                    &desc_sets,
                    &self.textures
                )
            };
        }
        unsafe { self.device.cmd_end_render_pass(command_buffer) };
        unsafe { self.device.end_command_buffer(command_buffer)}.unwrap();
        // self.primitive_pipeline.draw_frame(
        //     &self.device, command_buffer, current_frame, self.scene_descriptor_sets[current_frame], &self.mobjects, &mobj_desc_sets, &self.textures);

        let semaphores = vec![sync.image_available];
        let wait_stages = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = vec![sync.render_finished];
        let submit_info = SubmitInfo {
            s_type: StructureType::SUBMIT_INFO,
            wait_semaphore_count: 1,
            p_wait_semaphores: semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &command_buffer,
            signal_semaphore_count: 1,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        };
        unsafe {
            self.device
                .queue_submit(graphics_queue, &[submit_info], sync.in_flight)
        }
        .unwrap();
        let swapchains = vec![self.swapchain];
        let present_info = PresentInfoKHR {
            s_type: StructureType::PRESENT_INFO_KHR,
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            ..Default::default()
        };
        unsafe {
            self.swapchain_device
                .queue_present(present_queue, &present_info)
        }
        .unwrap();
    }
}

fn create_vk_instance(
    entry: &Entry,
    required_instance_extensions: Option<Vec<String>>,
) -> Instance {
    let app_name = CString::new("Mobject").unwrap();
    let engine_name = CString::new("No Engine").unwrap();

    let required_instance_extensions = required_instance_extensions.unwrap_or_default();
    let app_info = ApplicationInfo {
        s_type: StructureType::APPLICATION_INFO,
        p_application_name: app_name.as_ptr(),
        application_version: vk::make_api_version(1, 0, 0, 0),
        p_engine_name: engine_name.as_ptr(),
        engine_version: vk::make_api_version(1, 1, 1, 0),
        api_version: vk::make_api_version(0, 1, 1, 0),
        ..Default::default()
    };

    #[cfg(feature = "validation_layers")]
    let utf8_ptr = Utf8Pointer::new(&VALIDATION_LAYERS);
    let extensions_ptr = Utf8Pointer::new(&required_instance_extensions);
    let instance_info = InstanceCreateInfo {
        s_type: StructureType::INSTANCE_CREATE_INFO,
        p_application_info: &app_info,
        #[cfg(feature = "validation_layers")]
        enabled_layer_count: 1,
        #[cfg(feature = "validation_layers")]
        pp_enabled_layer_names: utf8_ptr.as_ptr(),
        enabled_extension_count: required_instance_extensions.len() as u32,
        pp_enabled_extension_names: extensions_ptr.as_ptr(),
        ..Default::default()
    };

    unsafe { entry.create_instance(&instance_info, None).unwrap() }
}
