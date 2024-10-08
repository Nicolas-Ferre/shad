use winit::platform::web::{EventLoopExtWebSys, WindowExtWebSys};

pub(crate) fn init_logging(level: log::Level) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = console_log::init_with_level(level);
}

pub(crate) fn init_canvas(handle: &winit::window::Window) {
    const CANVAS_ID: &str = "shad";
    if let Some(canvas) = handle.canvas() {
        canvas.set_id(CANVAS_ID);
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| body.append_child(&web_sys::Element::from(canvas)).ok())
            .expect("cannot append canvas to document body");
    }
}

pub(crate) fn run_event_loop(
    event_loop: winit::event_loop::EventLoop<()>,
    app: impl winit::application::ApplicationHandler + 'static,
) {
    event_loop.spawn_app(app);
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn surface_size(
    handle: &winit::window::Window,
    size: winit::dpi::PhysicalSize<u32>,
) -> winit::dpi::PhysicalSize<u32> {
    // If the size is not divided by the scale factor, then in case zoom is greater than 100%,
    // the canvas is recursively resized until reaching the maximum allowed size.
    let scale_factor = handle.scale_factor();
    winit::dpi::PhysicalSize::new(
        (f64::from(size.width) / scale_factor).round() as u32,
        (f64::from(size.height) / scale_factor).round() as u32,
    )
}

pub(crate) fn gpu_limits() -> wgpu::Limits {
    wgpu::Limits::downlevel_webgl2_defaults()
}
