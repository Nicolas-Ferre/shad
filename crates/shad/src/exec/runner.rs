use crate::exec::resources::ProgramResources;
use crate::exec::target::{Target, TargetConfig, TargetSpecialized, TextureTarget, WindowTarget};
use crate::exec::utils;
use crate::Program;
use futures::executor;
use std::sync::Arc;
use std::time::Instant;
use wgpu::{Adapter, Device, Instance, Queue, TextureViewDescriptor};
use winit::dpi::PhysicalSize;
use winit::window::Window;

#[derive(Debug)]
pub struct Runner {
    target: Target,
    instance: Instance,
    device: Device,
    adapter: Adapter,
    queue: Queue,
    resources: ProgramResources,
    frame_delta_secs: f32,
    last_frame_end: Instant,
}

impl Runner {
    /// Creates a new runner from a Shad program directory.
    pub fn new(program: Program, window: Option<Arc<Window>>, size: Option<(u32, u32)>) -> Self {
        executor::block_on(Self::new_async(program, window, size))
    }

    /// Creates a new runner from a Shad program directory.
    #[allow(clippy::future_not_send)]
    pub async fn new_async(
        program: Program,
        window: Option<Arc<Window>>,
        size: Option<(u32, u32)>,
    ) -> Self {
        let target = TargetConfig {
            size: size.unwrap_or((800, 600)),
        };
        let instance = utils::create_instance();
        let window_surface = window.map(|window| {
            // coverage: off (window cannot be tested)
            let surface = utils::create_surface(&instance, window.clone());
            (window, surface)
        }); // coverage: on
        let adapter = utils::create_adapter(
            &instance,
            window_surface.as_ref().map(|(_, surface)| surface),
        )
        .await;
        let (device, queue) = utils::create_device(&adapter).await;
        let surface_config = window_surface.as_ref().map(|(_, surface)| {
            // coverage: off (window cannot be tested)
            utils::create_surface_config(&adapter, &device, surface, target.size)
        }); // coverage: on
        let depth_buffer = utils::create_depth_buffer(&device, target.size);
        let target = if let (Some((window, surface)), Some(surface_config)) =
            (window_surface, surface_config)
        {
            // coverage: off (window cannot be tested)
            Target {
                inner: TargetSpecialized::Window(WindowTarget {
                    window,
                    surface,
                    surface_config,
                }),
                config: target,
                depth_buffer,
            }
            // coverage: on
        } else {
            let texture = utils::create_target_texture(&device, target.size);
            let view = texture.create_view(&TextureViewDescriptor::default());
            Target {
                inner: TargetSpecialized::Texture(TextureTarget { texture, view }),
                config: target,
                depth_buffer,
            }
        };
        let resources = ProgramResources::new(&device, program);
        Self {
            target,
            device,
            adapter,
            queue,
            resources,
            instance,
            frame_delta_secs: 0.,
            last_frame_end: Instant::now(),
        }
    }

    /// Returns compiled program details.
    pub fn program(&self) -> &Program {
        &self.resources.program
    }

    /// Returns the time of the last executed frame.
    pub fn delta_secs(&self) -> f32 {
        self.frame_delta_secs
    }

    /// Lists all GPU buffer names.
    pub fn buffers(&self) -> impl Iterator<Item = &String> {
        self.resources.program.buffers.keys()
    }

    /// Writes GPU buffer data.
    ///
    /// Buffer name includes the module path in which the module is defined
    /// (e.g. `inner.module.my_buffer`).
    ///
    /// # Panics
    ///
    /// This will panic if the `data` length doesn't match the buffer size.
    #[allow(clippy::cast_possible_truncation)]
    pub fn write(&self, buffer_name: &str, data: &[u8]) {
        if let (Some(buffer_props), Some(buffer)) = (
            self.resources.program.buffers.get(buffer_name),
            self.resources.buffers.get(buffer_name),
        ) {
            assert_eq!(
                data.len(),
                buffer_props.size_bytes as usize,
                "incorrect data size"
            );
            self.queue.write_buffer(buffer, 0, data);
        }
    }

    /// Reads GPU buffer data.
    ///
    /// Buffer name includes the module path in which the module is defined
    /// (e.g. `inner.module.my_buffer`).
    ///
    /// If the buffer doesn't exist, an empty vector is returned.
    pub fn read(&self, buffer_name: &str) -> Vec<u8> {
        if let (Some(buffer_props), Some(buffer)) = (
            self.resources.program.buffers.get(buffer_name),
            self.resources.buffers.get(buffer_name),
        ) {
            utils::read_buffer(&self.device, &self.queue, buffer, buffer_props.size_bytes)
        } else {
            vec![]
        }
    }

    /// Read texture target.
    ///
    /// If the surface is a window, an empty vector is returned.
    pub fn read_target(&self) -> Vec<u8> {
        match &self.target.inner {
            TargetSpecialized::Texture(target) => {
                utils::read_texture(&self.device, &self.queue, target, self.target.config.size)
            }
            TargetSpecialized::Window(_) => vec![], // no-coverage (window cannot be tested)
        }
    }

    /// Runs a step of the program.
    ///
    /// # Errors
    ///
    /// An error is returned if shader execution failed.
    pub fn run_step(&mut self) {
        let mut encoder = utils::create_encoder(&self.device);
        if self.resources.has_compute_step() {
            let pass = utils::start_compute_pass(&mut encoder);
            self.resources.run_compute_step(pass);
        }
        match &self.target.inner {
            // coverage: off (window cannot be tested)
            TargetSpecialized::Window(target) => {
                let texture = target.create_surface_texture();
                let view = utils::create_surface_view(&texture, target.surface_config.format);
                let pass =
                    utils::create_render_pass(&mut encoder, &view, &self.target.depth_buffer);
                drop(pass);
                self.queue.submit(Some(encoder.finish()));
                texture.present();
            }
            // coverage: on
            TargetSpecialized::Texture(target) => {
                let pass = utils::create_render_pass(
                    &mut encoder,
                    &target.view,
                    &self.target.depth_buffer,
                );
                drop(pass);
                self.queue.submit(Some(encoder.finish()));
            }
        }
        self.frame_delta_secs = self.last_frame_end.elapsed().as_secs_f32();
        self.last_frame_end = Instant::now();
    }

    // coverage: off (window cannot be tested)

    /// Requests window surface redraw.
    ///
    /// # Panics
    ///
    /// This will panic if the surface is not a window.
    pub fn request_redraw(&self) {
        match &self.target.inner {
            TargetSpecialized::Window(target) => target.window.request_redraw(),
            TargetSpecialized::Texture(_) => {
                unreachable!("surface should be a window")
            }
        }
    }

    /// Refreshes the rendering surface.
    pub fn refresh_surface(&mut self) {
        match &mut self.target.inner {
            TargetSpecialized::Window(target) => {
                target.surface = utils::create_surface(&self.instance, target.window.clone());
                target.surface_config = utils::create_surface_config(
                    &self.adapter,
                    &self.device,
                    &target.surface,
                    self.target.config.size,
                );
            }
            TargetSpecialized::Texture(_) => {
                unreachable!("internal error: refreshing non-window target surface")
            }
        }
    }

    /// Resizes rendering surface.
    pub fn update_surface_size(&mut self, size: PhysicalSize<u32>) {
        match &mut self.target.inner {
            TargetSpecialized::Window(target) => {
                self.target.config.size = (size.width.max(1), size.height.max(1));
                self.target.depth_buffer =
                    utils::create_depth_buffer(&self.device, self.target.config.size);
                target.surface_config = utils::create_surface_config(
                    &self.adapter,
                    &self.device,
                    &target.surface,
                    self.target.config.size,
                );
            }
            TargetSpecialized::Texture(_) => {
                unreachable!("internal error: updating non-window target surface")
            }
        }
    }

    // coverage: on
}
