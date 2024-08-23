use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, ComputePipeline,
    ComputePipelineDescriptor, DepthBiasState, DepthStencilState, Device, FragmentState, FrontFace,
    MultisampleState, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology,
    RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, StencilState, TextureFormat,
    VertexBufferLayout, VertexState,
};

#[derive(Debug)]
pub(crate) struct Shader {
    pub(crate) pipeline: RenderPipeline,
}

impl Shader {
    pub(crate) fn create_render(
        device: &Device,
        texture_format: TextureFormat,
        code: &str,
        buffer_layout: &[VertexBufferLayout<'_>],
    ) -> RenderPipeline {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("shad_shader"),
            source: wgpu::ShaderSource::Wgsl(code.into()),
        });
        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("shad_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("shad_render_pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &module,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: buffer_layout,
            },
            fragment: Some(FragmentState {
                module: &module,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: texture_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    }

    pub(crate) fn create_compute_pipeline(device: &Device, code: &str) -> ComputePipeline {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("shad_shader"),
            source: wgpu::ShaderSource::Wgsl(code.into()),
        });
        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &module,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    }
}
