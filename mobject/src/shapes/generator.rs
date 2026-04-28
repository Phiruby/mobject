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
            pub constraints: Vec<crate::physics::constraints::PhysicsConstraint>,
            com: Vec3,
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
                    constraints: Vec::new(),
                    ubo: UBO { model: glm::identity() },
                    texture_path: None,
                    pipeline: $pipeline,
                    com: glm::vec3(0.0, 0.0, 0.0),
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
                assert!(indices.len() % 3 == 0);
                assert!(self.vertices.len() > 0);
                Self {
                    indices,
                    ..self
                }
            }

            pub fn with_vertices(self, vertices: $vty) -> Self {
                let physics_vertices: Vec<crate::shapes::PhysicsVertex> = vertices.iter().map(|v| crate::shapes::PhysicsVertex::new(v.position)).collect();
                // TODO: initial center of mass different from next
                // iteration's computation (if you add constraints)
                // fix this up!
                // dbg!(&vertices);
                let com = crate::physics::center_of_mass(&physics_vertices);
                // dbg!(com);
                let physics_vertices = physics_vertices
                    .into_iter()
                    .map(|v| {
                        let pos = v.position.clone();
                        v.with_body_space_position(pos - com)
                    })
                    .collect();
                Self {
                    vertices,
                    physics_vertices,
                    com,
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

            pub fn with_constraints(self, constraints: Vec<crate::physics::constraints::PhysicsConstraint>) -> Self {
                // TODO: static / attachment constraints should get infinite mass
                Self {
                    constraints,
                    ..self
                }
            }

            pub fn with_inverse_mass(self, inv_masses: Vec<f32>) -> Self {
                let mut pv = self.physics_vertices;
                for (v, m) in pv.iter_mut().zip(inv_masses.iter()) {
                    v.w = *m;
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
            // constraint generation
            pub fn make_static(mut self) -> Self {
                let mut new_constraints = Vec::new();
                for (i, v) in self.vertices.iter_mut().enumerate() {
                    new_constraints.push(
                        crate::physics::constraints::PhysicsConstraint::Static(
                            crate::physics::constraints::StaticConstraint {
                                pin_to: v.position,
                                inp_vertex_index: i
                            }
                        )
                    );
                    self.physics_vertices[i].w = 0.0;
                }
                let cm = crate::physics::center_of_mass(&self.physics_vertices);
                let old_cm = self.com;
                self.physics_vertices.iter_mut().for_each(|v| v.body_space_position += old_cm - cm);
                Self {
                    constraints: new_constraints,
                    com: cm,
                    ..self
                }
            }
            /// Pins the vertices indexed by `indices`
            pub fn anchor(mut self, indices: Vec<usize>) -> Self {
                for i in indices {
                    self.constraints.push(
                        crate::physics::constraints::PhysicsConstraint::Static(
                            crate::physics::constraints::StaticConstraint {
                                pin_to: self.vertices[i].position,
                                inp_vertex_index: i
                            }
                        )
                    );
                    self.physics_vertices[i].w = 0.0;
                }
                let cm = crate::physics::center_of_mass(&self.physics_vertices);
                let old_cm = self.com;
                self.physics_vertices.iter_mut().for_each(|v| v.body_space_position += old_cm - cm);
                Self {
                    constraints: self.constraints,
                    com: cm,
                    ..self
                }
            }
            /// Turns the obj into a rigid body: meaning distances
            /// between vertices are preserved. If you anchor a few vertices
            /// without calling `rigid_body`, you'll notice the object
            /// "drop and expand" due to gravity
            pub fn rigid_body(mut self) -> Self {
                self.constraints.push(
                    crate::physics::constraints::PhysicsConstraint::RigidBody(
                        crate::physics::constraints::ShapeConstraint()
                    )
                );
                self
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

            fn get_mut_vertices_and_constraints(&mut self) -> (&mut [crate::shapes::PhysicsVertex], &[crate::physics::constraints::PhysicsConstraint]) {
                (&mut self.physics_vertices, &self.constraints)
            }

            fn get_physics_vertices(&self) -> &[crate::shapes::PhysicsVertex] {
                &self.physics_vertices
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
            fn sync_phys_and_render_vertices(&mut self) {
                self.vertices
                    .iter_mut()
                    .zip(self.physics_vertices.iter())
                    .for_each(|(v, pv)| {
                        // TODO: also compute new normal
                        v.position = pv.position;
                    });
            }
            fn set_com(&mut self, com: Vec3) {
                self.com = com
            }

            fn update_spatial_hash(&self, space: &mut shapeject::SpatialHash3D<Vec<(u32, usize)>>, mid: u32) {
                let triple_min = |x: f32, y: f32, z: f32| -> f32 { x.min(y).min(z) };
                let triple_max = |x: f32, y: f32, z: f32| -> f32 { x.max(y).max(z) };
                for (j, i) in (0..self.indices.len()).step_by(3).enumerate() {
                    let (i1, i2, i3) = (self.indices[i], self.indices[i + 1], self.indices[i + 2]);
                    let (v1, v2, v3) = (&self.physics_vertices[i1 as usize], &self.physics_vertices[i2 as usize], &self.physics_vertices[i3 as usize]);
                    let mx = triple_min(v1.position.x, v2.position.x, v3.position.x);
                    let my = triple_min(v1.position.y, v2.position.y, v3.position.y);
                    let mz = triple_min(v1.position.z, v2.position.z, v3.position.z);
                    let hx = triple_max(v1.position.x, v2.position.x, v3.position.x);
                    let hy = triple_max(v1.position.y, v2.position.y, v3.position.y);
                    let hz = triple_max(v1.position.z, v2.position.z, v3.position.z);
                    space.iter_cubes_mut(cgmath::Vector3::new(mx, my, mz), cgmath::Vector3::new(hx, hy, hz)).for_each(|(_, _, ent)| {ent.push((mid, j))});
                }
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
