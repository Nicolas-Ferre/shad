use crate::error::Error;
use futures::executor;
use shad_analyzer::AnalyzedProgram;
use shad_parser::ParsedProgram;
use std::path::Path;
use wgpu::{
    Adapter, Backends, BindGroup, Buffer, BufferDescriptor, BufferUsages, CommandEncoder,
    CommandEncoderDescriptor, ComputePass, ComputePassDescriptor, ComputePipeline,
    ComputePipelineDescriptor, Device, DeviceDescriptor, Features, Gles3MinorVersion, Instance,
    InstanceFlags, Limits, MapMode, MemoryHints, PowerPreference, Queue, RequestAdapterOptions,
    ShaderModuleDescriptor,
};

/// A runner to execute a Shad script.
#[derive(Debug)]
pub struct Runner {
    device: Device,
    queue: Queue,
    program: Program,
}

impl Runner {
    /// Initializes a runner for a Shad script located at a specific `path`.
    ///
    /// # Errors
    ///
    /// An error if the Shad script cannot be compiled.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let instance = Self::create_instance();
        let adapter = Self::create_adapter(&instance);
        let (device, queue) = Self::create_device(&adapter);
        let program = Program::new(path, &device)?;
        Ok(Self {
            device,
            queue,
            program,
        })
    }

    /// Starts the runner.
    pub fn run(&self) {
        self.program.init(&self.device, &self.queue);
    }

    /// Retrieves the bytes of the buffer with a specific Shad `name`.
    pub fn buffer(&self, name: &str) -> Vec<u8> {
        if let Some(&index) = self.program.analyzed.buffers.buffer_name_indexes.get(name) {
            let size = self.program.analyzed.buffers.buffers[index].type_.size as u64;
            let tmp_buffer = self.device.create_buffer(&BufferDescriptor {
                label: Some("modor_texture_buffer"),
                size,
                usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("shad:buffer_retrieval"),
                });
            encoder.copy_buffer_to_buffer(&self.program.buffers[index], 0, &tmp_buffer, 0, size);
            let submission_index = self.queue.submit(Some(encoder.finish()));
            let slice = tmp_buffer.slice(..);
            slice.map_async(MapMode::Read, |_| ());
            self.device
                .poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));
            let view = slice.get_mapped_range();
            let content = view.to_vec();
            drop(view);
            tmp_buffer.unmap();
            content
        } else {
            vec![]
        }
    }

    fn create_instance() -> Instance {
        Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or_else(Backends::all),
            flags: InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            gles_minor_version: Gles3MinorVersion::Automatic,
        })
    }

    fn create_adapter(instance: &Instance) -> Adapter {
        let adapter_request = RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        };
        executor::block_on(instance.request_adapter(&adapter_request))
            .expect("no supported graphic adapter found")
    }

    fn create_device(adapter: &Adapter) -> (Device, Queue) {
        let device_descriptor = DeviceDescriptor {
            label: None,
            required_features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            required_limits: Limits::default(),
            memory_hints: MemoryHints::Performance,
        };
        executor::block_on(adapter.request_device(&device_descriptor, None))
            .expect("error when retrieving graphic device")
    }
}

#[derive(Debug)]
struct Program {
    analyzed: AnalyzedProgram,
    buffers: Vec<Buffer>,
    init_compute_shaders: Vec<ComputeShader>,
}

impl Program {
    fn new(path: impl AsRef<Path>, device: &Device) -> Result<Self, Error> {
        let parsed = ParsedProgram::parse_file(path).map_err(|err| match err {
            shad_parser::Error::Syntax(err) => Error::Syntax(err),
            shad_parser::Error::Io(err) => Error::Io(err),
        })?;
        let analyzed = AnalyzedProgram::analyze(&parsed);
        if analyzed.errors().next().is_some() {
            return Err(Error::Semantic(analyzed.errors().cloned().collect()));
        }
        let buffers: Vec<_> = analyzed
            .buffers
            .buffers
            .iter()
            .map(|buffer| Self::create_buffer(buffer, device))
            .collect();
        let init_compute_shaders = analyzed
            .init_compute_shaders
            .shaders
            .iter()
            .map(|shader| ComputeShader::new(shader, &buffers, device))
            .collect();
        Ok(Self {
            analyzed,
            buffers,
            init_compute_shaders,
        })
    }

    fn init(&self, device: &Device, queue: &Queue) {
        let mut encoder = Self::create_encoder(device);
        let mut pass = Self::start_compute_pass(&mut encoder);
        for shader in &self.init_compute_shaders {
            pass.set_pipeline(&shader.pipeline);
            pass.set_bind_group(0, &shader.bind_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        drop(pass);
        queue.submit(Some(encoder.finish()));
    }

    fn create_buffer(buffer: &shad_analyzer::Buffer, device: &Device) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some(&format!("shad:buffer:{}", buffer.name.label)),
            size: buffer.type_.size as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }

    fn create_encoder(device: &Device) -> CommandEncoder {
        device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("shad:encoder"),
        })
    }

    pub(crate) fn start_compute_pass(encoder: &mut CommandEncoder) -> ComputePass<'_> {
        encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        })
    }
}

#[derive(Debug)]
struct ComputeShader {
    pipeline: ComputePipeline,
    bind_group: BindGroup,
}

impl ComputeShader {
    fn new(shader: &shad_analyzer::ComputeShader, buffers: &[Buffer], device: &Device) -> Self {
        let pipeline = Self::create_pipeline(shader, device);
        let bind_group = Self::create_bind_group(&pipeline, shader, buffers, device);
        Self {
            pipeline,
            bind_group,
        }
    }

    fn create_pipeline(shader: &shad_analyzer::ComputeShader, device: &Device) -> ComputePipeline {
        let code = shad_transpiler::generate_wgsl_compute_shader(shader);
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("shad_shader"),
            source: wgpu::ShaderSource::Wgsl(code.into()),
        });
        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&format!("shad:compute_shader:{}", shader.name)),
            layout: None,
            module: &module,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group(
        pipeline: &ComputePipeline,
        shader: &shad_analyzer::ComputeShader,
        buffers: &[Buffer],
        device: &Device,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &shader
                .buffers
                .iter()
                .enumerate()
                .map(|(index, buffer)| wgpu::BindGroupEntry {
                    binding: index as u32,
                    resource: buffers[buffer.index].as_entire_binding(),
                })
                .collect::<Vec<_>>(),
        })
    }
}