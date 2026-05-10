#[macro_export]
macro_rules! define_shape {
    (
        $vis:vis struct $name:ident {
            vertices: Vec<RenderVertex> $(,)?,
            indices:  $ity:ty $(,)?
        },
        $pipeline:expr,
        $manifold:expr
    ) => {
        $vis struct $name {
            pub vertices: Vec<RenderVertex>,
            pub indices:  $ity,
            physics_start_index: usize,
            physics_total_vertices: usize,
            com: Vec3,
            texture_path: Option<String>,
            uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
            uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
            uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
            ubo: UBO,
            pipeline: Pipelines,
            manifold: crate::shapes::Manifold,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    vertices: Vec::new().try_into().unwrap(),
                    indices: Vec::new(),
                    physics_start_index: 0,
                    physics_total_vertices: 0,
                    uniform_buffers: None,
                    uniform_buffer_memories: None,
                    uniform_mapped_memories: None,
                    ubo: UBO { model: glm::identity() },
                    texture_path: None,
                    pipeline: $pipeline,
                    manifold: $manifold,
                    com: glm::vec3(0.0, 0.0, 0.0),
                }
            }
            pub fn finish_construction(self) -> crate::shapes::ShapeIntent {
                crate::shapes::ShapeIntent::new(Box::new(self))
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
                assert!(indices.len() % 3 == 0);
                assert!(self.vertices.len() > 0);
                Self {
                    indices,
                    ..self
                }
            }

            pub fn with_vertices(self, vertices: Vec<RenderVertex>) -> Self {
                Self {
                    vertices,
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
            fn get_vertices(&self) -> &[RenderVertex] {
                &self.vertices
            }
        }

        impl crate::shapes::ShapeConstruction for $name {
            fn get_vertices(&self) -> &[RenderVertex] {
                &self.vertices
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
            fn sync_phys_and_render_vertices(&mut self, physics_vertices: &[crate::shapes::PhysicsVertex]) {
                let np = (0..self.physics_total_vertices).map(|i| physics_vertices[i + self.physics_start_index].position).collect::<Vec<Vec3>>();
                let normals = crate::shapes::compute_normals(
                    &self.indices.iter().map(|i| *i as usize).collect::<Vec<usize>>(),
                    &np,
                );
                for i in 0..self.physics_total_vertices {
                    self.vertices[i as usize].position = physics_vertices[i + self.physics_start_index as usize].position;
                    self.vertices[i as usize].normal = normals[i as usize];
                }
            }
            fn set_com(&mut self, com: Vec3) {
                self.com = com
            }

            fn update_spatial_hash(&self, space: &mut shapeject::SpatialHash3D<Vec<(u32, usize)>>, mid: u32) {
                let triple_min = |x: f32, y: f32, z: f32| -> f32 { x.min(y).min(z) };
                let triple_max = |x: f32, y: f32, z: f32| -> f32 { x.max(y).max(z) };
                for (j, i) in (0..self.indices.len()).step_by(3).enumerate() {
                    let (i1, i2, i3) = (self.indices[i], self.indices[i + 1], self.indices[i + 2]);
                    let (v1, v2, v3) = (&self.vertices[i1 as usize], &self.vertices[i2 as usize], &self.vertices[i3 as usize]);
                    let mx = triple_min(v1.position.x, v2.position.x, v3.position.x);
                    let my = triple_min(v1.position.y, v2.position.y, v3.position.y);
                    let mz = triple_min(v1.position.z, v2.position.z, v3.position.z);
                    let hx = triple_max(v1.position.x, v2.position.x, v3.position.x);
                    let hy = triple_max(v1.position.y, v2.position.y, v3.position.y);
                    let hz = triple_max(v1.position.z, v2.position.z, v3.position.z);
                    space.iter_cubes_mut(cgmath::Vector3::new(mx, my, mz), cgmath::Vector3::new(hx, hy, hz)).for_each(|(_, _, ent)| {ent.push((mid, j))});
                }
            }
            fn set_physics_index_range(&mut self, start: usize, total: usize) {
                self.physics_start_index = start;
                self.physics_total_vertices = total;
            }
            fn get_physics_index_range(&self) -> (usize, usize) { (self.physics_start_index, self.physics_total_vertices) }
            fn manifold(&self) -> crate::shapes::Manifold { self.manifold }
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
