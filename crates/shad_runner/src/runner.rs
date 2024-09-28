use futures::executor;
use fxhash::FxHashMap;
use shad_analyzer::{Asg, AsgBuffer, AsgComputeShader};
use shad_error::Error;
use shad_parser::Ast;
use std::path::Path;
use std::time::{Duration, Instant};
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
    is_started: bool,
    last_delta: Duration,
    last_step_end: Instant,
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

    /// Retrieves the bytes of the buffer with a specific Shad `name`.
    pub fn buffer(&self, name: &str) -> Vec<u8> {
        if let (Some(asg_buffer), Some(wgpu_buffer)) = (
            self.program.asg.buffers.get(name),
            self.program.buffers.get(name),
        ) {
            let size = asg_buffer
                .expr
                .as_ref()
                .expect("internal error: invalid expr")
                .type_(&self.program.asg)
                .expect("internal error: invalid type")
                .size as u64;
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
    asg: Asg,
    buffers: FxHashMap<String, Buffer>,
    init_shaders: Vec<ComputeShader>,
    step_shaders: Vec<ComputeShader>,
}

impl Program {
    #[allow(clippy::similar_names)]
    fn new(path: impl AsRef<Path>, device: &Device) -> Result<Self, Error> {
        let ast = Ast::from_file(path)?;
        let asg = Asg::analyze(&ast);
        if !asg.errors.is_empty() {
            return Err(Error::Semantic(asg.errors));
        }
        let buffers: FxHashMap<_, _> = asg
            .buffers
            .iter()
            .map(|(name, buffer)| (name.clone(), Self::create_buffer(&asg, buffer, device)))
            .collect();
        let init_shaders = asg
            .init_shaders
            .iter()
            .map(|shader| ComputeShader::new(&asg, shader, &buffers, device))
            .collect();
        let step_shaders = asg
            .step_shaders
            .iter()
            .map(|shader| ComputeShader::new(&asg, shader, &buffers, device))
            .collect();
        Ok(Self {
            asg,
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

    fn create_buffer(asg: &Asg, buffer: &AsgBuffer, device: &Device) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some(&format!("shad:buffer:{}", buffer.ast.name.label)),
            size: buffer
                .expr
                .as_ref()
                .expect("internal error: invalid expr")
                .type_(asg)
                .expect("internal error: invalid type")
                .size as u64,
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
    bind_group: Option<BindGroup>,
}

impl ComputeShader {
    fn new(
        asg: &Asg,
        shader: &AsgComputeShader,
        buffers: &FxHashMap<String, Buffer>,
        device: &Device,
    ) -> Self {
        let pipeline = Self::create_pipeline(asg, shader, device);
        let bind_group = (!shader.buffers.is_empty())
            .then(|| Self::create_bind_group(&pipeline, shader, buffers, device));
        Self {
            pipeline,
            bind_group,
        }
    }

    fn create_pipeline(asg: &Asg, shader: &AsgComputeShader, device: &Device) -> ComputePipeline {
        let code = shad_transpiler::generate_wgsl_compute_shader(asg, shader)
            .expect("internal error: invalid shader code");
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
        shader: &AsgComputeShader,
        buffers: &FxHashMap<String, Buffer>,
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
                    resource: buffers[&buffer.ast.name.label].as_entire_binding(),
                })
                .collect::<Vec<_>>(),
        })
    }
}
