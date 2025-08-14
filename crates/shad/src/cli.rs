#![allow(clippy::print_stdout, clippy::use_debug)]

use crate::{compilation, Runner};
use clap::Parser;
use futures::channel::oneshot::{Receiver, Sender};
use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

// coverage: off (not easy to test)

#[cfg(target_os = "android")]
pub(crate) static ANDROID_APP: std::sync::OnceLock<android_activity::AndroidApp> =
    std::sync::OnceLock::new();

/// Arguments of `shad` CLI.
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub enum Args {
    /// Run a Shad program.
    Run(RunArgs),
}

impl Args {
    /// Runs CLI depending on provided arguments.
    pub fn run(self) {
        match self {
            Self::Run(args) => args.run(),
        }
    }
}

#[doc(hidden)]
#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct RunArgs {
    /// Path to the Shad program directory to run.
    pub path: PathBuf,
    /// List of GPU buffers to display at each step.
    #[arg(short, long, num_args(0..), default_values_t = Vec::<String>::new())]
    pub buffer: Vec<String>,
    /// Print FPS in standard output.
    #[clap(long, short, action)]
    pub fps: bool,
}

impl RunArgs {
    const DEFAULT_SIZE: (u32, u32) = (800, 600);

    fn run(self) {
        let path = self.path.clone();
        let mut runner = WindowRunner::new(self, move |event_loop, sender| {
            let window = Self::create_window(event_loop, Self::DEFAULT_SIZE);
            let program = match compilation::compile(path.as_path()) {
                Ok(program) => program,
                Err(err) => {
                    eprintln!("{}", err.render());
                    process::exit(1);
                }
            };
            sender
                .send(Runner::new(program, Some(window), None))
                .expect("Cannot send created runner");
        });
        EventLoop::builder()
            .build()
            .expect("event loop initialization failed")
            .run_app(&mut runner)
            .expect("event loop failed");
    }

    /// Runs a Shad program on Web.
    #[cfg(target_arch = "wasm32")]
    pub fn run_web(self, source: impl crate::SourceFolder + Send + 'static) {
        use winit::platform::web::{EventLoopExtWebSys, WindowExtWebSys};
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        let _ = console_log::init_with_level(log::Level::Info);
        let program = match compilation::compile(source) {
            Ok(program) => program,
            Err(err) => {
                log::error!("{}", err.render());
                return;
            }
        };
        let runner = WindowRunner::new(self, move |event_loop, sender| {
            let window = Self::create_window(event_loop, Self::DEFAULT_SIZE);
            if let Some(canvas) = window.canvas() {
                canvas.set_id("shad");
                web_sys::window()
                    .and_then(|win| win.document())
                    .and_then(|doc| doc.body())
                    .and_then(|body| body.append_child(&web_sys::Element::from(canvas)).ok())
                    .expect("cannot append canvas to document body");
            }
            wasm_bindgen_futures::spawn_local(async move {
                sender
                    .send(Runner::new_async(program, Some(window), None).await)
                    .expect("Cannot send created runner");
            });
        });
        EventLoop::builder()
            .build()
            .expect("event loop initialization failed")
            .spawn_app(runner);
    }

    /// Runs a Shad program on Android.
    #[cfg(target_os = "android")]
    pub fn run_android(
        self,
        android_app: android_activity::AndroidApp,
        source: impl crate::SourceFolder + Send + 'static,
    ) {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        let program = match compilation::compile(source) {
            Ok(program) => program,
            Err(err) => {
                eprintln!("{}", err.render());
                process::exit(1);
            }
        };
        let mut runner = WindowRunner::new(self, move |event_loop, sender| {
            let window = Self::create_window(event_loop, Self::DEFAULT_SIZE);
            sender
                .send(Runner::new(program, Some(window), None))
                .expect("Cannot send created runner");
        });
        ANDROID_APP.get_or_init(|| android_app.clone());
        EventLoop::builder()
            .with_android_app(android_app)
            .build()
            .expect("event loop initialization failed")
            .run_app(&mut runner)
            .expect("event loop failed");
    }

    fn create_window(event_loop: &ActiveEventLoop, size: (u32, u32)) -> Arc<Window> {
        let size = PhysicalSize::new(size.0, size.1);
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("")
                    .with_inner_size(size),
            )
            .expect("cannot create window");
        Arc::new(window)
    }
}

struct WindowRunner {
    args: RunArgs,
    #[allow(clippy::type_complexity)]
    create_runner_fn: Option<Box<dyn FnOnce(&ActiveEventLoop, Sender<Runner>)>>,
    runner: Option<Runner>,
    runner_receiver: Option<Receiver<Runner>>,
}

impl ApplicationHandler for WindowRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.refresh_surface(event_loop);
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(receiver) = &mut self.runner_receiver {
            if let Ok(Some(runner)) = receiver.try_recv() {
                self.runner = Some(runner);
                self.runner_receiver = None;
            }
        }
        if self.runner.is_some() {
            match event {
                WindowEvent::RedrawRequested => self.update(),
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => self.update_window_size(size),
                _ => (),
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(runner) = &mut self.runner {
            runner.request_redraw();
        }
    }
}

impl WindowRunner {
    fn new(
        args: RunArgs,
        create_runner_fn: impl FnOnce(&ActiveEventLoop, Sender<Runner>) + 'static,
    ) -> Self {
        Self {
            args,
            create_runner_fn: Some(Box::new(create_runner_fn)),
            runner: None,
            runner_receiver: None,
        }
    }

    fn refresh_surface(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(runner) = &mut self.runner {
            runner.refresh_surface();
        } else {
            let (sender, receiver) = futures::channel::oneshot::channel();
            let create_runner_id = self
                .create_runner_fn
                .take()
                .expect("internal error: missing runner init closure");
            self.runner_receiver = Some(receiver);
            create_runner_id(event_loop, sender);
        }
    }

    fn update(&mut self) {
        if let Some(runner) = &mut self.runner {
            runner.run_step();
            if self.args.fps {
                println!("FPS: {}", (1. / runner.delta_secs()).round());
            }
            for buffer in &self.args.buffer {
                println!("Buffer `{buffer}`: {:?}", runner.read(buffer));
            }
        }
    }

    fn update_window_size(&mut self, size: PhysicalSize<u32>) {
        if let Some(runner) = &mut self.runner {
            runner.update_surface_size(size);
        }
    }
}
