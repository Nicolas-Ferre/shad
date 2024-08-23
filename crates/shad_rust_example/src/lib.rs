#![allow(missing_docs)]

use futures::executor;
use log::{error, trace, Level};
use std::sync::Arc;
use wgpu::{
    Adapter, Backends, Color, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor,
    Extent3d, Features, Gles3MinorVersion, Instance, InstanceFlags, LoadOp, MemoryHints,
    Operations, PowerPreference, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp,
    Surface, SurfaceConfiguration, SurfaceTexture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

mod platform;
mod validation;

pub fn run() {
    platform::init_logging(Level::Info);
    let event_loop = platform::event_loop();
    let app = App::default();
    platform::run_event_loop(event_loop, app);
}

#[derive(Debug, Default)]
struct App {
    ctx: Option<Context>,
    is_suspended: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init_surface(event_loop);
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => self.update(),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.update_window_size(size),
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ctx) = &mut self.ctx {
            ctx.window.request_redraw();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.is_suspended = true;
    }
}

impl App {
    const DEFAULT_SIZE: (u32, u32) = (800, 600);

    fn init_surface(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(ctx) = &mut self.ctx {
            ctx.surface = Self::create_surface(&ctx.instance, ctx.window.clone());
            ctx.adapter = Self::create_adapter(&ctx.instance, &ctx.surface);
            (ctx.device, ctx.queue) = Self::create_device(&ctx.adapter);
            ctx.surface_config =
                Self::create_surface_config(&ctx.adapter, &ctx.device, &ctx.surface, ctx.size);
        } else {
            let size = Self::DEFAULT_SIZE;
            let instance = Self::create_instance();
            let window = Self::create_window(event_loop);
            let surface = Self::create_surface(&instance, window.clone());
            let adapter = Self::create_adapter(&instance, &surface);
            let (device, queue) = Self::create_device(&adapter);
            let surface_config = Self::create_surface_config(&adapter, &device, &surface, size);
            let depth_buffer = Self::create_depth_buffer(&device, size);
            self.ctx = Some(Context {
                instance,
                adapter,
                device,
                queue,
                window,
                surface,
                surface_config,
                size,
                depth_buffer,
            });
        }
    }

    fn update_window_size(&mut self, size: PhysicalSize<u32>) {
        if let Some(ctx) = &mut self.ctx {
            ctx.size = (size.width, size.height);
            ctx.depth_buffer = Self::create_depth_buffer(&ctx.device, ctx.fix_surface_size());
            ctx.surface_config =
                Self::create_surface_config(&ctx.adapter, &ctx.device, &ctx.surface, ctx.size);
        }
    }

    fn update(&mut self) {
        self.is_suspended = false;
        if let Some(ctx) = &mut self.ctx {
            ctx.render();
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

    fn create_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
        let window = event_loop
            .create_window(
                Window::default_attributes().with_inner_size(PhysicalSize::new(
                    Self::DEFAULT_SIZE.0,
                    Self::DEFAULT_SIZE.1,
                )),
            )
            .expect("cannot create window");
        platform::init_canvas(&window);
        Arc::new(window)
    }

    fn create_surface(instance: &Instance, window: Arc<Window>) -> Surface<'static> {
        instance
            .create_surface(window)
            .expect("cannot create surface")
    }

    fn create_surface_config(
        adapter: &Adapter,
        device: &Device,
        surface: &Surface<'_>,
        size: (u32, u32),
    ) -> SurfaceConfiguration {
        let config = surface
            .get_default_config(adapter, size.0, size.1)
            .expect("internal error: not supported surface");
        surface.configure(device, &config);
        config
    }

    fn create_adapter(instance: &Instance, surface: &Surface<'_>) -> Adapter {
        let adapter_request = RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        };
        executor::block_on(instance.request_adapter(&adapter_request))
            .expect("no supported graphic adapter found")
    }

    fn create_device(adapter: &Adapter) -> (Device, Queue) {
        let device_descriptor = DeviceDescriptor {
            label: None,
            required_features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            required_limits: platform::gpu_limits(),
            memory_hints: MemoryHints::Performance,
        };
        executor::block_on(adapter.request_device(&device_descriptor, None))
            .expect("error when retrieving graphic device")
    }

    fn create_depth_buffer(device: &Device, size: (u32, u32)) -> TextureView {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("modor_depth_texture"),
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
}

#[derive(Debug)]
struct Context {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    window: Arc<Window>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    size: (u32, u32),
    depth_buffer: TextureView,
}

impl Context {
    fn fix_surface_size(&self) -> (u32, u32) {
        let size = PhysicalSize::new(self.size.0, self.size.1);
        let surface_size = platform::surface_size(&self.window, size);
        (surface_size.width, surface_size.height)
    }

    fn render(&self) {
        let texture = self.create_surface_texture();
        let view = Self::create_surface_view(&texture);
        let mut encoder = self.create_encoder();
        let pass = self.create_pass(&mut encoder, &view);
        let result = validation::validate_wgpu(&self.device, false, || drop(pass));
        let is_err = result.is_err();
        if let Err(err) = result {
            error!("Error during target rendering: {err}");
        } else {
            self.queue.submit(Some(encoder.finish()));
            trace!("Window target rendered (error: {})", is_err);
        }
        texture.present();
    }

    fn create_surface_texture(&self) -> SurfaceTexture {
        self.surface
            .get_current_texture()
            .expect("internal error: cannot retrieve surface texture")
    }

    fn create_surface_view(texture: &SurfaceTexture) -> TextureView {
        texture
            .texture
            .create_view(&TextureViewDescriptor::default())
    }

    fn create_encoder(&self) -> CommandEncoder {
        let descriptor = CommandEncoderDescriptor {
            label: Some("modor_render_encoder"),
        };
        self.device.create_command_encoder(&descriptor)
    }

    fn create_pass<'a>(
        &self,
        encoder: &'a mut CommandEncoder,
        view: &'a TextureView,
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("modor_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_buffer,
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
}
