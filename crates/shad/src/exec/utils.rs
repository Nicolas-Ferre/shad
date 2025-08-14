use crate::exec::target::TextureTarget;
use std::sync::Arc;
use wgpu::{
    Adapter, BackendOptions, Backends, BindGroupLayout, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferUsages, Color, CommandEncoder,
    CommandEncoderDescriptor, CompositeAlphaMode, ComputePass, ComputePassDescriptor,
    ComputePipeline, ComputePipelineDescriptor, Device, DeviceDescriptor, Extent3d, Features,
    Instance, InstanceFlags, Limits, LoadOp, MapMode, MemoryBudgetThresholds, MemoryHints,
    Operations, PipelineCompilationOptions, PipelineLayoutDescriptor, PollType, PowerPreference,
    Queue, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    RenderPassDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderStages, StoreOp,
    Surface, SurfaceConfiguration, SurfaceTexture, TexelCopyBufferInfo, TexelCopyBufferLayout,
    Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor, Trace,
};
use winit::window::Window;

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn padded_unpadded_row_bytes(width: u32) -> (u32, u32) {
    let bytes_per_pixel = size_of::<u32>() as u32;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
    (
        unpadded_bytes_per_row + padded_bytes_per_row_padding,
        unpadded_bytes_per_row,
    )
}

pub(crate) fn create_instance() -> Instance {
    Instance::new(&wgpu::InstanceDescriptor {
        backends: Backends::from_env().unwrap_or_else(Backends::all),
        flags: InstanceFlags::default(),
        memory_budget_thresholds: MemoryBudgetThresholds::default(),
        backend_options: BackendOptions::default(),
    })
}

#[allow(clippy::future_not_send)]
pub(crate) async fn create_adapter(
    instance: &Instance,
    window_surface: Option<&Surface<'_>>,
) -> Adapter {
    let adapter_request = RequestAdapterOptions {
        power_preference: PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: window_surface,
    };
    instance
        .request_adapter(&adapter_request)
        .await
        .expect("no supported graphic adapter found")
}

#[allow(clippy::future_not_send)]
pub(crate) async fn create_device(adapter: &Adapter) -> (Device, Queue) {
    let device_descriptor = DeviceDescriptor {
        label: Some("shad:device"),
        required_features: Features::default(),
        required_limits: Limits::default(),
        memory_hints: MemoryHints::Performance,
        trace: Trace::Off,
    };
    adapter
        .request_device(&device_descriptor)
        .await
        .expect("error when retrieving graphic device")
}

pub(crate) fn create_buffer(device: &Device, label: &str, size: u64) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some(label),
        size,
        usage: BufferUsages::STORAGE
            | BufferUsages::COPY_SRC
            | BufferUsages::COPY_DST
            | BufferUsages::UNIFORM
            | BufferUsages::VERTEX
            | BufferUsages::INDEX,
        mapped_at_creation: false,
    })
}

pub(crate) fn create_bind_group_layout(
    device: &Device,
    stages: ShaderStages,
    storage_count: u32,
) -> Option<BindGroupLayout> {
    if storage_count == 0 {
        None
    } else {
        Some(
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shad:bing_group_layout"),
                entries: &(0..storage_count)
                    .map(|binding| BindGroupLayoutEntry {
                        binding,
                        visibility: stages,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    })
                    .collect::<Vec<_>>(),
            }),
        )
    }
}

pub(crate) fn create_encoder(device: &Device) -> CommandEncoder {
    device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("shad:encoder"),
    })
}

pub(crate) fn start_compute_pass(encoder: &mut CommandEncoder) -> ComputePass<'_> {
    encoder.begin_compute_pass(&ComputePassDescriptor {
        label: Some("shad:compute_pass"),
        timestamp_writes: None,
    })
}

pub(crate) fn create_target_texture(device: &Device, size: (u32, u32)) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some("shad:target_texture"),
        size: Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

pub(crate) fn create_depth_buffer(device: &Device, size: (u32, u32)) -> TextureView {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("shad:depth_texture"),
        size: Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    texture.create_view(&TextureViewDescriptor::default())
}

pub(crate) fn create_render_pass<'a>(
    encoder: &'a mut CommandEncoder,
    view: &'a TextureView,
    depth_buffer: &'a TextureView,
) -> RenderPass<'a> {
    encoder.begin_render_pass(&RenderPassDescriptor {
        label: Some("shad:render_pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
            view: depth_buffer,
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    })
}

pub(crate) fn create_compute_pipeline(
    device: &Device,
    layout: &BindGroupLayout,
    code: &str,
) -> ComputePipeline {
    let module = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("shad:shader_module"),
        source: wgpu::ShaderSource::Wgsl(code.into()),
    });
    device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("shad:compute_pipeline"),
        layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("shad:compute_pipeline_layout"),
            bind_group_layouts: &[layout],
            push_constant_ranges: &[],
        })),
        module: &module,
        entry_point: None,
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    })
}

// coverage: off (window cannot be tested)

pub(crate) fn create_surface(instance: &Instance, window: Arc<Window>) -> Surface<'static> {
    instance
        .create_surface(window)
        .expect("cannot create surface")
}

pub(crate) fn create_surface_config(
    adapter: &Adapter,
    device: &Device,
    surface: &Surface<'_>,
    size: (u32, u32),
) -> SurfaceConfiguration {
    let format = surface.get_capabilities(adapter).formats[0];
    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        #[cfg(target_os = "android")]
        format: format.add_srgb_suffix(),
        #[cfg(not(target_os = "android"))]
        format: format.remove_srgb_suffix(),
        width: size.0,
        height: size.1,
        present_mode: surface.get_capabilities(adapter).present_modes[0],
        desired_maximum_frame_latency: 2,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![format.add_srgb_suffix()],
    };
    surface.configure(device, &config);
    config
}

pub(crate) fn create_surface_view(texture: &SurfaceTexture, format: TextureFormat) -> TextureView {
    texture.texture.create_view(&TextureViewDescriptor {
        format: Some(format.add_srgb_suffix()),
        ..Default::default()
    })
}

pub(crate) fn read_buffer(device: &Device, queue: &Queue, buffer: &Buffer, size: u64) -> Vec<u8> {
    let read_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("shad:buffer:storage_read"),
        size,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = create_encoder(device);
    encoder.copy_buffer_to_buffer(buffer, 0, &read_buffer, 0, Some(size));
    let submission_index = queue.submit(Some(encoder.finish()));
    let slice = read_buffer.slice(..);
    slice.map_async(MapMode::Read, |_| ());
    device
        .poll(PollType::WaitForSubmissionIndex(submission_index))
        .expect("cannot read buffer");
    let view = slice.get_mapped_range();
    let content = view.to_vec();
    drop(view);
    read_buffer.unmap();
    content
}

pub(crate) fn read_texture(
    device: &Device,
    queue: &Queue,
    target: &TextureTarget,
    size: (u32, u32),
) -> Vec<u8> {
    let (padded_row_bytes, _) = padded_unpadded_row_bytes(size.0);
    let tmp_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("shad:buffer:target_read"),
        size: (padded_row_bytes * size.1).into(),
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = create_encoder(device);
    encoder.copy_texture_to_buffer(
        target.texture.as_image_copy(),
        TexelCopyBufferInfo {
            buffer: &tmp_buffer,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row_bytes),
                rows_per_image: None,
            },
        },
        Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
    );
    let submission_index = queue.submit(Some(encoder.finish()));
    let slice = tmp_buffer.slice(..);
    slice.map_async(MapMode::Read, |_| ());
    device
        .poll(PollType::WaitForSubmissionIndex(submission_index))
        .expect("cannot read target buffer");
    let view = slice.get_mapped_range();
    let (padded_row_bytes, unpadded_row_bytes) = padded_unpadded_row_bytes(size.0);
    let content = view
        .chunks(padded_row_bytes as usize)
        .flat_map(|a| &a[..unpadded_row_bytes as usize])
        .copied()
        .collect();
    drop(view);
    tmp_buffer.unmap();
    content
}

// coverage: on
