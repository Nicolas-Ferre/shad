use bytemuck::Pod;
use std::marker::PhantomData;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindGroup, BufferUsages, ComputePipeline, Device, Queue};

#[derive(Debug)]
pub(crate) struct Buffer<T> {
    pub(crate) inner: wgpu::Buffer,
    pub(crate) len: usize,
    usage: BufferUsages,
    phantom: PhantomData<fn(T)>,
}

impl<T> Buffer<T>
where
    T: Pod,
{
    pub(crate) fn new(device: &Device, usage: BufferUsages, data: &[T]) -> Self {
        Self {
            inner: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("shad_buffer"),
                contents: bytemuck::cast_slice(data),
                usage,
            }),
            len: data.len(),
            usage,
            phantom: PhantomData,
        }
    }

    pub(crate) fn update(&mut self, device: &Device, queue: &Queue, data: &[T]) {
        if self.len < data.len() {
            *self = Self::new(device, self.usage, data);
        } else {
            queue.write_buffer(&self.inner, 0, bytemuck::cast_slice(data));
        }
    }
}

#[derive(Debug)]
pub(crate) struct BufferBindGroup {
    pub(crate) inner: BindGroup,
}

impl BufferBindGroup {
    pub(crate) fn new<T>(
        device: &Device,
        compute_shader: &ComputePipeline,
        bind_group: u32,
        buffer: &Buffer<T>,
    ) -> Self {
        Self {
            inner: device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &compute_shader.get_bind_group_layout(bind_group),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.inner.as_entire_binding(),
                }],
            }),
        }
    }
}
