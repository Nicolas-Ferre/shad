use futures::executor;
use fxhash::FxHashMap;
use shad_analyzer::{Analysis, BufferId, ComputeShader};
use shad_error::Error;
use shad_parser::Ast;
use std::iter;
use std::path::Path;
use std::time::{Duration, Instant};
use wgpu::{
    Adapter, Backends, BindGroup, BufferDescriptor, BufferUsages, CommandEncoder,
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
    is_started: bool,
    last_delta: Duration,
    last_step_end: Instant,
}

impl Runner {
    /// Initializes a runner for a Shad script located at a specific `path`.
    ///
    /// `path` can be:
    /// - a folder: the file `main.shd` at the root of the folder will be taken as entrypoint.
    /// - a file: the file will be taken as entry point.
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
            is_started: false,
            last_delta: Duration::ZERO,
            last_step_end: Instant::now(),
        })
    }

    /// Runs a step of the application.
    pub fn run_step(&mut self) {
        let start = Instant::now();
        if !self.is_started {
            self.program.init(&self.device, &self.queue);
            self.is_started = true;
        }
        self.program.run_step(&self.device, &self.queue);
        let end = Instant::now();
        self.last_delta = end - start;
        self.last_step_end = end;
    }

    /// Returns the time taken by the latest step.
    pub fn delta(&self) -> Duration {
        self.last_delta
    }

    /// Returns the analyzed code.
    pub fn analysis(&self) -> &Analysis {
        &self.program.analysis
    }

    /// Retrieves the bytes of the buffer with a specific Shad `name`.
    pub fn buffer(&self, buffer_id: &BufferId) -> Vec<u8> {
        if let Some(wgpu_buffer) = self.program.buffers.get(buffer_id) {
            let type_ = self
                .program
                .analysis
                .buffer_type(buffer_id)
                .expect("internal error: invalid buffer type");
            let size = type_.size.into();
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
            encoder.copy_buffer_to_buffer(wgpu_buffer, 0, &tmp_buffer, 0, size);
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
    analysis: Analysis,
    buffers: FxHashMap<BufferId, wgpu::Buffer>,
    init_shaders: Vec<RunComputeShader>,
    step_shaders: Vec<RunComputeShader>,
}

impl Program {
    #[allow(clippy::similar_names)]
    fn new(path: impl AsRef<Path>, device: &Device) -> Result<Self, Error> {
        let path = path.as_ref();
        let asts = if path.is_dir() {
            Ast::from_dir(path)?
        } else {
            iter::once(("main".to_string(), Ast::from_file(path, "main")?)).collect()
        };
        let analysis = Analysis::run(asts);
        if !analysis.errors.is_empty() {
            return Err(Error::Semantic(analysis.errors));
        }
        let buffers: FxHashMap<_, _> = analysis
            .buffers
            .keys()
            .map(|id| (id.clone(), Self::create_buffer(&analysis, id, device)))
            .collect();
        let init_shaders = analysis
            .init_shaders
            .iter()
            .map(|shader| RunComputeShader::new(&analysis, shader, &buffers, device))
            .collect();
        let step_shaders = analysis
            .step_shaders
            .iter()
            .map(|shader| RunComputeShader::new(&analysis, shader, &buffers, device))
            .collect();
        Ok(Self {
            analysis,
            buffers,
            init_shaders,
            step_shaders,
        })
    }

    fn init(&self, device: &Device, queue: &Queue) {
        let mut encoder = Self::create_encoder(device);
        let mut pass = Self::start_compute_pass(&mut encoder);
        for shader in &self.init_shaders {
            pass.set_pipeline(&shader.pipeline);
            if let Some(bind_group) = &shader.bind_group {
                pass.set_bind_group(0, bind_group, &[]);
            }
            pass.dispatch_workgroups(1, 1, 1);
        }
        drop(pass);
        queue.submit(Some(encoder.finish()));
    }

    fn run_step(&self, device: &Device, queue: &Queue) {
        let mut encoder = Self::create_encoder(device);
        let mut pass = Self::start_compute_pass(&mut encoder);
        for shader in &self.step_shaders {
            pass.set_pipeline(&shader.pipeline);
            if let Some(bind_group) = &shader.bind_group {
                pass.set_bind_group(0, bind_group, &[]);
            }
            pass.dispatch_workgroups(1, 1, 1);
        }
        drop(pass);
        queue.submit(Some(encoder.finish()));
    }

    fn create_buffer(analysis: &Analysis, buffer: &BufferId, device: &Device) -> wgpu::Buffer {
        let type_ = analysis
            .buffer_type(buffer)
            .expect("internal error: invalid buffer type");
        device.create_buffer(&BufferDescriptor {
            label: Some(&format!("shad:buffer:{}.{}", buffer.module, buffer.name)),
            size: type_.size.into(),
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
struct RunComputeShader {
    pipeline: ComputePipeline,
    bind_group: Option<BindGroup>,
}

impl RunComputeShader {
    fn new(
        analysis: &Analysis,
        shader: &ComputeShader,
        buffers: &FxHashMap<BufferId, wgpu::Buffer>,
        device: &Device,
    ) -> Self {
        let pipeline = Self::create_pipeline(analysis, shader, device);
        let bind_group = (!shader.buffer_ids.is_empty())
            .then(|| Self::create_bind_group(&pipeline, shader, buffers, device));
        Self {
            pipeline,
            bind_group,
        }
    }

    fn create_pipeline(
        analysis: &Analysis,
        shader: &ComputeShader,
        device: &Device,
    ) -> ComputePipeline {
        let code = shad_transpiler::generate_wgsl_compute_shader(analysis, shader);
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("shad_shader"),
            source: wgpu::ShaderSource::Wgsl(code.into()),
        });
        device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("shad:compute_shader"),
            layout: None,
            module: &module,
            entry_point: None,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group(
        pipeline: &ComputePipeline,
        shader: &ComputeShader,
        buffers: &FxHashMap<BufferId, wgpu::Buffer>,
        device: &Device,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &shader
                .buffer_ids
                .iter()
                .enumerate()
                .map(|(index, buffer)| wgpu::BindGroupEntry {
                    binding: index as u32,
                    resource: buffers[buffer].as_entire_binding(),
                })
                .collect::<Vec<_>>(),
        })
    }
}
