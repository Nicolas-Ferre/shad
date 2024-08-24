use crate::app::Gpu;
use crate::buffer::Buffer;
use crate::mesh::{Mesh, Vertex};
use crate::shader;
use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BufferAddress, BufferUsages, ComputePipeline, Device, IndexFormat, RenderPipeline,
    TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};

const SPRITE_COUNT: u32 = 1_000_000;

#[derive(Debug)]
pub(crate) struct Program {
    rendering_shader: RenderPipeline,
    update_shader: ComputePipeline,
    mesh: Mesh,
    instances: Buffer<Sprite>,
    updated_instances_bind_group: BindGroup,
    delta: Buffer<f32>,
    delta_bind_group: BindGroup,
}

impl Program {
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn new(device: &Device, texture_format: TextureFormat) -> Self {
        let rendering_shader = Self::rendering_shader(device, texture_format);
        let update_shader = Self::update_shader(device);
        let mesh = Mesh::rectangle(device);
        let mut sprites = vec![];
        for i in 0..SPRITE_COUNT {
            sprites.push(Sprite {
                position: [
                    i as f32 / SPRITE_COUNT as f32 * 0.5,
                    i as f32 / SPRITE_COUNT as f32 * -0.5,
                ],
                size: [0.05, 0.05],
                rotation: 0.,
                _padding3: [0.],
                _padding4: [0., 0.],
                color: [1., 1., 1., 1.],
            });
        }
        let instances = Buffer::new(
            device,
            BufferUsages::VERTEX
                | BufferUsages::STORAGE
                | BufferUsages::COPY_SRC
                | BufferUsages::COPY_DST,
            &sprites,
        );
        let updated_instances_bind_group = instances.create_bind_group(device, &update_shader, 0);
        let delta = Buffer::new(
            device,
            BufferUsages::UNIFORM | BufferUsages::STORAGE | BufferUsages::COPY_DST,
            &[0.],
        );
        let delta_bind_group = delta.create_bind_group(device, &update_shader, 1);
        Self {
            rendering_shader,
            update_shader,
            mesh,
            instances,
            updated_instances_bind_group,
            delta,
            delta_bind_group,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn update(&mut self, gpu: &mut Gpu<'_>) {
        self.delta.update(gpu.device, gpu.queue, &[gpu.delta * 2.]);
        let mut pass = gpu.start_compute_pass();
        pass.set_pipeline(&self.update_shader);
        pass.set_bind_group(0, &self.updated_instances_bind_group, &[]);
        pass.set_bind_group(1, &self.delta_bind_group, &[]);
        pass.dispatch_workgroups((self.instances.len as u32).div_ceil(256), 1, 1);
        drop(pass);
        let mut pass = gpu.start_render_pass();
        pass.set_pipeline(&self.rendering_shader);
        pass.set_index_buffer(self.mesh.index_buffer.inner.slice(..), IndexFormat::Uint16);
        pass.set_vertex_buffer(0, self.mesh.vertex_buffer.inner.slice(..));
        pass.set_vertex_buffer(1, self.instances.inner.slice(..));
        pass.draw_indexed(
            0..(self.mesh.index_buffer.len as u32),
            0,
            0..self.instances.len as u32,
        );
    }

    fn rendering_shader(device: &Device, texture_format: TextureFormat) -> RenderPipeline {
        shader::create_render_pipeline(
            device,
            texture_format,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/res/sprite_rendering.wgsl"
            )),
            &[
                VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    }],
                },
                VertexBufferLayout {
                    array_stride: size_of::<Sprite>() as BufferAddress,
                    step_mode: VertexStepMode::Instance,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 1,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 2,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32,
                            offset: 16,
                            shader_location: 3,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32,
                            offset: 20,
                            shader_location: 5,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 24,
                            shader_location: 6,
                        },
                        VertexAttribute {
                            format: VertexFormat::Float32x4,
                            offset: 32,
                            shader_location: 4,
                        },
                    ],
                },
            ],
        )
    }

    fn update_shader(device: &Device) -> ComputePipeline {
        shader::create_compute_pipeline(
            device,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/res/sprite_update.wgsl"
            )),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Sprite {
    position: [f32; 2],
    size: [f32; 2],
    rotation: f32,
    _padding3: [f32; 1], // WGSL requires storage/uniform data aligned on 16 bytes
    _padding4: [f32; 2], // WGSL requires storage/uniform data aligned on 16 bytes
    color: [f32; 4],
}
