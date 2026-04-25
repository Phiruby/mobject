#[macro_export]
macro_rules! define_shape {
    (
        $vis:vis struct $name:ident {
            vertices: $vty:ty,
            indices:  $ity:ty $(,)?
        },
        $pipeline:expr
    ) => {
        $vis struct $name {
            pub vertices: $vty,
            pub indices:  $ity,
            pub physics_vertices: Vec<crate::shapes::PhysicsVertex>,
            texture_path: Option<String>,
            uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
            uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
            uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
            ubo: UBO,
            pipeline: Pipelines
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    vertices: Vec::new().try_into().unwrap(),
                    indices: Vec::new(),
                    physics_vertices: Vec::new(),
                    uniform_buffers: None,
                    uniform_buffer_memories: None,
                    uniform_mapped_memories: None,
                    ubo: UBO { model: glm::identity() },
                    texture_path: None,
                    pipeline: $pipeline
                }
            }

            pub fn with_ubo(self, ubo: UBO) -> Self {
                Self {
                    ubo,
                    ..self
                }
            }

            pub fn with_texture(self, image_path: String) -> Self {
                Self {
                    texture_path: Some(image_path),
                    ..self
                }
            }

            pub fn with_indices(self, indices: $ity) -> Self {
                Self {
                    indices,
                    ..self
                }
            }

            pub fn with_vertices(self, vertices: $vty) -> Self {
                let physics_vertices = vertices.iter().map(|v| crate::shapes::PhysicsVertex::new(v.position)).collect();
                Self {
                    vertices,
                    physics_vertices,
                    ..self
                }
            }

            pub fn with_velocities(self, velocities: Vec<Vec3>) -> Self {
                let mut pv = self.physics_vertices;
                for (v, vel) in pv.iter_mut().zip(velocities.iter()) {
                    v.velocity = *vel;
                }
                Self {
                    physics_vertices: pv,
                    ..self
                }
            }

            pub fn include_texture(self, texture_path: &str) -> Self {
                Self {
                    texture_path: Some(String::from(texture_path)),
                    vertices: self.vertices
                    .iter()
                    .map(|v|
                        RenderVertex::with_tex_coord(
                            v.position,
                            v.normal,
                            v.tex_coord
                        )
                    )
                    .collect::<Vec<RenderVertex>>()
                    .try_into()
                    .unwrap(),
                    ..self
                }
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

            fn texture_path(&self) -> Option<&str> {
                self.texture_path.as_deref()
            }
        }

        impl crate::shapes::ShapeConstruction for $name {
            fn get_vertices(&self) -> &[RenderVertex] {
                &self.vertices
            }

            fn get_mut_vertices(&mut self) -> &mut [crate::shapes::PhysicsVertex] {
                &mut self.physics_vertices
            }

            fn get_pipeline(&self) -> Pipelines {
                self.pipeline
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
            fn texture_path(&self) -> Option<&str> {
                self.texture_path.as_deref()
            }
        }
        impl crate::shapes::ShapeMotion for $name {
            fn rotate(&mut self, axis: &glm::Vec3, angle: f32) {
                let model_matrix = self.ubo.model;
                let new_model_matrix = glm::rotate(&model_matrix, angle, axis);
                self.ubo.model = new_model_matrix;
            }
            fn get_ubo_contents(&self) -> &UBO {
                &self.ubo
            }
        }
        impl BuiltShape for $name {}
    };
}
