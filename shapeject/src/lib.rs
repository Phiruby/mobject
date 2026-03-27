macro_rules! define_shape {
    (
        $vis:vis struct $name:ident {
            vertices: $vty:ty,
            indices:  $ity:ty $(,)?
        }
    ) => {
        $vis struct $name {
            pub vertices: $vty,
            pub indices:  $ity,

            uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
            uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
            uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
            ubo: UBO,
        }

        impl $name {
            pub fn with_ubo(vertices: $vty, indices: $ity, ubo: UBO) -> Self {
                Self {
                    vertices,
                    indices,
                    uniform_buffers: None,
                    uniform_buffer_memories: None,
                    uniform_mapped_memories: None,
                    ubo,
                }
            }

            pub fn with_indices(vertices: $vty, indices: $ity) -> Self {
                Self::with_ubo(
                    vertices,
                    indices,
                    UBO { model: glm::identity() },
                )
            }

            pub fn new(vertices: $vty) -> Self {
                Self::with_ubo(
                    vertices,
                    (0..vertices.len()).collect(),
                    UBO { model: glm::identity() },
                )
            }
        }

        impl Shape for $name {
            fn build(
                self: Box<Self>,
                device: &Device,
                mem_properties: PhysicalDeviceMemoryProperties,
            ) -> Box<dyn BuiltShape> {
                let (uniform_buffers, uniform_buffer_memories, uniform_mapped_memories) =
                    buffers::create_uniform_buffers::<{ MAX_FRAMES_IN_FLIGHT as usize }>(
                        device,
                        mem_properties,
                        std::mem::size_of::<UBO>() as u64,
                    );

                let mut this = *self;
                this.uniform_buffers = Some(uniform_buffers);
                this.uniform_buffer_memories = Some(uniform_buffer_memories);
                this.uniform_mapped_memories = Some(uniform_mapped_memories);

                Box::new(this)
            }
        }

        impl BuiltShape for $name {
            fn get_vertices(&self) -> &[Vertex2D] {
                &self.vertices
            }

            fn indices(&self) -> &[u32] {
                &self.indices
            }

            fn get_uniform_buffer(
                &self,
            ) -> (
                &[Buffer; MAX_FRAMES_IN_FLIGHT as usize],
                &[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize],
                &[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize],
            ) {
                (
                    self.uniform_buffers.as_ref().unwrap(),
                    self.uniform_buffer_memories.as_ref().unwrap(),
                    self.uniform_mapped_memories.as_ref().unwrap(),
                )
            }

            fn get_ubo_contents(&self) -> &UBO {
                &self.ubo
            }
        }
    };
}
