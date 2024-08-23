use crate::buffer::Buffer;
use wgpu::{BufferUsages, Device};

#[derive(Debug)]
pub(crate) struct Mesh {
    pub(crate) vertex_buffer: Buffer<Vertex>,
    pub(crate) index_buffer: Buffer<u16>,
}

impl Mesh {
    pub(crate) fn rectangle(device: &Device) -> Self {
        Self {
            vertex_buffer: Buffer::new(
                device,
                BufferUsages::VERTEX,
                &[
                    Vertex {
                        position: [-0.5, 0.5, 0.],
                    },
                    Vertex {
                        position: [-0.5, -0.5, 0.],
                    },
                    Vertex {
                        position: [0.5, -0.5, 0.],
                    },
                    Vertex {
                        position: [0.5, 0.5, 0.],
                    },
                ],
            ),
            index_buffer: Buffer::new(device, BufferUsages::INDEX, &[0, 1, 2, 0, 2, 3]),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Vertex {
    position: [f32; 3],
}
