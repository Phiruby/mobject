use ash::vk::{
    self, ApplicationInfo, Buffer, CommandBuffer, CommandBufferResetFlags, DescriptorPool, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorType, DeviceMemory, Extent2D, Fence, Framebuffer, Handle, Image, ImageAspectFlags, ImageView, InstanceCreateInfo, MemoryPropertyFlags, PhysicalDeviceFeatures, PhysicalDeviceMemoryProperties, Pipeline, PipelineLayout, PresentInfoKHR, Queue, RenderPass, Sampler, ShaderStageFlags, StructureType, SubmitInfo, SurfaceKHR, SwapchainKHR
};
use ash::{Device, Entry, Instance, khr, khr::surface};
use crate::c_utils::Utf8Pointer;
use crate::device::QueueFamilies;
use glfw::PWindow;
use nalgebra_glm as glm;
use crate::shapes::{BuiltShape, GlobalUBO, Shape, UBO};
use std::ffi::{CString, c_void};
use crate::{window, swapchain, shaders, render_pass, buffers, texture, shapes, device};
use std::time::{Instant, Duration};
use crate::MAX_FRAMES_IN_FLIGHT;
const VALIDATION_LAYERS: [&str; 1] = ["VK_LAYER_KHRONOS_validation"];
const DEVICE_EXTENSIONS: [&str; 1] = ["VK_KHR_swapchain"];

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

pub struct Scene {
    state: SceneState,
    mobjects: Vec<Box<dyn BuiltShape>>,
    actions: Vec<Action>,
    sync: Vec<window::Sync>,
    device: Device,
    swapchain_device: khr::swapchain::Device,
    window: PWindow,
    swapchain: SwapchainKHR,
    command_buffer: Vec<CommandBuffer>,
    render_pass: RenderPass,
    framebuffers: Vec<Framebuffer>,
    vertex_buffers: Vec<Buffer>,
    vertex_buffer_memory: Vec<DeviceMemory>,
    index_buffers: Vec<Buffer>,
    index_buffer_memory: Vec<DeviceMemory>,
    _image: Image,
    _image_memory: DeviceMemory,
    uniform_buffers: Vec<Buffer>,
    uniform_buffer_memories: Vec<DeviceMemory>,
    uniform_buffer_mapped_memories: Vec<*mut c_void>,
    texture_image_view: ImageView,
    depth_image: Image,
    depth_image_view: ImageView,
    depth_image_memory: DeviceMemory,
    texture_sampler: Sampler,
    scene_descriptor_sets: [DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize],
    // TODO: move descriptor sets in a struct with the actual mobject
    mobject_descriptor_sets: Vec<[DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize]>,
    mobject_descriptor_pool: DescriptorPool,
    mobject_descriptor_set_layout: DescriptorSetLayout,
    extent: Extent2D,
    graphics_pipeline: Pipeline,
    queue_families: QueueFamilies,
    global_ubo: GlobalUBO,
    pipeline_layout: PipelineLayout,
    physical_device_properties: PhysicalDeviceMemoryProperties
}

impl Scene {
    pub fn new(mobjects: Option<Vec<Box<dyn Shape>>>) -> Self {
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
            // NOTE: this is needed for texture anisotropy sampling
            Some(PhysicalDeviceFeatures {
                sampler_anisotropy: vk::TRUE,
                tessellation_shader: vk::TRUE,
                ..Default::default()
            }),
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
        let (image, image_memory, mip_levels) = texture::create_texture_image(
            &logical_device,
            "textures/viking_room.png",
            physical_device_memory_properties,
            pool,
            graphics_queue,
        );
        let texture_image_view =
            texture::create_texture_image_view(&logical_device, image, mip_levels);
        let sampler = texture::create_sampler(&logical_device, &instance, physical_device);
        let (vertex_buffers, vertex_buffer_memories) = buffers::create_vertex_buffers(
            &logical_device,
            60_000,
            physical_device_memory_properties,
        );
        let (index_buffers, index_buffer_memory) = buffers::create_index_buffers(
            &logical_device,
            60_000,
            physical_device_memory_properties,
        );
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
        let scene_descriptor_set_layout = shaders::create_description_set_layout(
            &logical_device,
            [DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: ShaderStageFlags::VERTEX | ShaderStageFlags::TESSELLATION_EVALUATION,
                ..Default::default()
            }]
            .to_vec(),
        );
        let scene_descriptor_pool = shaders::scene_descriptor_pool(&logical_device);
        let scene_descriptor_sets = shaders::scene_descriptor_sets(
            &logical_device,
            scene_descriptor_set_layout,
            scene_descriptor_pool,
            &uniform_buffers,
        );

        let mobjects: Vec<Box<dyn BuiltShape>> = mobjects
            .unwrap_or_default()
            .into_iter()
            .map(|obj| obj.build(&logical_device, physical_device_memory_properties))
            .collect();

        let mobject_descriptor_set_layout = shaders::create_description_set_layout(
            &logical_device,
            [
                DescriptorSetLayoutBinding {
                    binding: 0,
                    descriptor_type: DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: 1,
                    stage_flags: ShaderStageFlags::VERTEX
                        | ShaderStageFlags::TESSELLATION_EVALUATION,
                    ..Default::default()
                },
                DescriptorSetLayoutBinding {
                    binding: 1,
                    descriptor_type: DescriptorType::COMBINED_IMAGE_SAMPLER,
                    descriptor_count: 1,
                    stage_flags: ShaderStageFlags::FRAGMENT,
                    ..Default::default()
                },
            ]
            .to_vec(),
        );
        // TODO: unhardcode 10
        let mobject_descriptor_pool =
            shaders::mobject_descriptor_pool(&logical_device, 10 as u32);
        let mobject_descriptor_sets: Vec<[DescriptorSet; MAX_FRAMES_IN_FLIGHT as usize]> = mobjects
            .iter()
            .map(|mob| {
                let (uniform_buffer, _, _) = mob.get_uniform_buffer();
                shaders::mobject_descriptor_sets(
                    &logical_device,
                    mobject_descriptor_set_layout,
                    mobject_descriptor_pool,
                    uniform_buffer,
                    texture_image_view,
                    sampler,
                )
            })
            .collect();
        let (graphics_pipeline, pipeline_layout) = shaders::create_graphics_pipeline(
            &logical_device,
            extent,
            render_pass,
            scene_descriptor_set_layout,
            mobject_descriptor_set_layout,
        );
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
            framebuffers,
            vertex_buffers,
            vertex_buffer_memory: vertex_buffer_memories,
            index_buffers,
            index_buffer_memory,
            texture_sampler: sampler,
            uniform_buffers: uniform_buffers.to_vec(),
            uniform_buffer_memories: uniform_buffer_memories.to_vec(),
            _image: image,
            _image_memory: image_memory,
            texture_image_view,
            depth_image,
            depth_image_view,
            depth_image_memory,
            uniform_buffer_mapped_memories: uniform_buffer_mapped_memories.to_vec(),
            scene_descriptor_sets,
            mobject_descriptor_sets,
            mobject_descriptor_pool,
            mobject_descriptor_set_layout,
            extent,
            graphics_pipeline,
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
            pipeline_layout,
            physical_device_properties: physical_device_memory_properties
        }
    }

    pub fn add(&mut self, mobject: Box<dyn Shape>) {
        self.actions.push(Action::AddMobject(mobject));
    }
    pub fn wait(&mut self, seconds: u8) {
        self.actions.push(Action::Wait { seconds });
    }

    fn set_state(&mut self, state: SceneState) {
        self.state = state;
    }

    fn take_action(&mut self) {
        if self.actions.len() == 0 {return ;}
        // NOTE: going backwards. doing this for now for simplicity
        let action = self.actions.pop().unwrap();
        match action {
            Action::AddMobject(mobj) => {
                dbg!("Adding to mobjects");
                self.mobjects.push(mobj.build(&self.device, self.physical_device_properties));
                let (uniform_buffer, _, _) = self.mobjects.last().unwrap().get_uniform_buffer();
                self.mobject_descriptor_sets.push(
                    shaders::mobject_descriptor_sets(
                        &self.device,
                        self.mobject_descriptor_set_layout,
                        self.mobject_descriptor_pool,
                        uniform_buffer,
                        self.texture_image_view,
                        self.texture_sampler,
                    )
                );
            },
            Action::Wait { seconds } => {dbg!("Waiting..."); self.set_state(SceneState::Waiting{ from: Instant::now(), duration: Duration::from_secs(seconds as u64) })},
        }
    }

    pub fn main_loop(&mut self) {
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

    fn draw_frame(&mut self, graphics_queue: Queue, present_queue: Queue, current_frame: usize) {
        let vertex_buffer = self.vertex_buffers[current_frame];
        let vertex_buffer_memory = self.vertex_buffer_memory[current_frame];
        let index_buffer = self.index_buffers[current_frame];
        let index_buffer_memory = self.index_buffer_memory[current_frame];
        let sync = &self.sync[current_frame];
        let command_buffer = self.command_buffer[current_frame];

        unsafe {
            self.device
                .wait_for_fences(&[sync.in_flight], true, u64::MAX)
        }
        .unwrap();
        unsafe { self.device.reset_fences(&[sync.in_flight]) }.unwrap();
        let (vertices, indices) = shapes::mobjects_to_vertices_and_indices(&self.mobjects);
        window::fill_vertex_buffer(&self.device, vertex_buffer_memory, &vertices);
        window::fill_index_buffer(&self.device, index_buffer_memory, &indices);
        window::fill_uniform_buffer(
            self.uniform_buffer_mapped_memories[current_frame],
            &self.global_ubo,
        );

        self.mobjects.iter().for_each(|mob| {
            let (_, _, mapped_memory) = mob.get_uniform_buffer();
            window::fill_uniform_buffer(mapped_memory[current_frame], mob.get_ubo_contents());
        });
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

        buffers::record_command_buffer(
            &self.device,
            command_buffer,
            vertex_buffer,
            index_buffer,
            image_index,
            self.render_pass,
            &self.framebuffers,
            self.scene_descriptor_sets[current_frame],
            self.mobject_descriptor_sets
                .iter()
                .map(|dsets| dsets[current_frame])
                .collect(),
            &self.mobjects,
            self.extent,
            self.pipeline_layout,
            self.graphics_pipeline,
        );
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
