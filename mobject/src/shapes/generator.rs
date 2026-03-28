use ash::vk::{Buffer, DeviceMemory};
use ash::Device;
use crate::MAX_FRAMES_IN_FLIGHT;
use crate::buffers;
use crate::shapes::{UBO, Shape, BuiltShape, Vertex2D, ShapeConstruction, ShapeMotion};
use nalgebra_glm as glm;
use crate::pipelines::Pipelines;
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
            texture_path: Option<String>,
            uniform_buffers: Option<[Buffer; MAX_FRAMES_IN_FLIGHT as usize]>,
            uniform_buffer_memories: Option<[DeviceMemory; MAX_FRAMES_IN_FLIGHT as usize]>,
            uniform_mapped_memories: Option<[*mut c_void; MAX_FRAMES_IN_FLIGHT as usize]>,
            ubo: UBO,
            pipeline: Pipelines
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
                    texture_path: None,
                    pipeline: $pipeline
                }
            }

            pub fn with_texture(vertices: $vty, indices: $ity, image_path: String) -> Self {
                let mut me = Self::with_ubo(
                    vertices,
                    indices,
                    UBO { model: glm::identity() },
                );
                me.texture_path = Some(image_path);
                me
            }

            pub fn with_indices(vertices: $vty, indices: $ity) -> Self {
                Self::with_ubo(
                    vertices,
                    indices,
                    UBO { model: glm::identity() },
                )
            }

            pub fn new(vertices: $vty) -> Self {
                let nvertices: u32 = vertices.len() as u32;
                Self::with_ubo(
                    vertices,
                    (0..nvertices).collect(),
                    UBO { model: glm::identity() },
                )
            }

            pub fn include_texture(self, texture_path: &str) -> Self {
                Self {
                    texture_path: Some(String::from(texture_path)),
                    vertices: self.vertices
                    .iter()
                    .map(|v|
                        Vertex2D::with_tex_coord(
                            v.position,
                            v.tex_coord
                        )
                    )
                    .collect::<Vec<Vertex2D>>()
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
            fn get_vertices(&self) -> &[Vertex2D] {
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
