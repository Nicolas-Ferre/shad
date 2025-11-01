use crate::exec::utils;
use crate::{Program, Shader};
use std::collections::HashMap;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, ComputePass, ComputePipeline, Device, ShaderStages,
};

#[derive(Debug)]
pub(crate) struct ProgramResources {
    pub(crate) program: Program,
    pub(crate) buffers: HashMap<String, Buffer>,
    compute_shaders: Vec<ComputeShader>,
}

impl ProgramResources {
    pub(crate) fn new(device: &Device, program: Program) -> Self {
        let buffers = program
            .buffers
            .iter()
            .map(|(name, buffer)| {
                (
                    name.clone(),
                    utils::create_buffer(device, name, buffer.size_bytes),
                )
            })
            .collect();
        let init_shaders = program
            .init_shaders
            .iter()
            .filter_map(|shader| ComputeShader::new(device, &buffers, shader, true));
        let run_shaders = program
            .run_shaders
            .iter()
            .filter_map(|shader| ComputeShader::new(device, &buffers, shader, false));
        let compute_shaders = init_shaders.chain(run_shaders).collect();
        Self {
            program,
            buffers,
            compute_shaders,
        }
    }

    pub(crate) fn has_compute_step(&self) -> bool {
        self.compute_shaders.iter().any(ComputeShader::should_run)
    }

    pub(crate) fn run_compute_step(&mut self, mut pass: ComputePass<'_>) {
        for shader in &mut self.compute_shaders {
            if shader.should_run() {
                pass.set_pipeline(&shader.pipeline);
                pass.set_bind_group(0, &shader.bind_group, &[]);
                pass.dispatch_workgroups(1, 1, 1);
                shader.is_init_done = true;
            }
        }
    }
}

#[derive(Debug)]
struct ComputeShader {
    pub(crate) pipeline: ComputePipeline,
    pub(crate) bind_group: BindGroup,
    pub(crate) is_init: bool,
    pub(crate) is_init_done: bool,
}

impl ComputeShader {
    #[allow(clippy::cast_possible_truncation)]
    fn new(
        device: &Device,
        buffers: &HashMap<String, Buffer>,
        shader: &Shader,
        is_init: bool,
    ) -> Option<Self> {
        let layout = utils::create_bind_group_layout(
            device,
            ShaderStages::COMPUTE,
            shader.buffers.len() as u32,
        )?;
        let pipeline = utils::create_compute_pipeline(device, &layout, &shader.code);
        let bind_group = Self::create_bind_group(device, &layout, shader, buffers);
        Some(Self {
            pipeline,
            bind_group,
            is_init,
            is_init_done: false,
        })
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group(
        device: &Device,
        layout: &BindGroupLayout,
        shader: &Shader,
        buffers: &HashMap<String, Buffer>,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shad:bind_group"),
            layout,
            entries: &shader
                .buffers
                .iter()
                .enumerate()
                .map(|(binding, name)| wgpu::BindGroupEntry {
                    binding: binding as u32,
                    resource: buffers[name].as_entire_binding(),
                })
                .collect::<Vec<_>>(),
        })
    }

    fn should_run(&self) -> bool {
        !self.is_init || !self.is_init_done
    }
}
