use winit::platform::android::EventLoopBuilderExtAndroid;

#[doc(hidden)]
pub(crate) static ANDROID_APP: std::sync::OnceLock<android_activity::AndroidApp> =
    std::sync::OnceLock::new();

pub(crate) fn event_loop() -> winit::event_loop::EventLoop<()> {
    let android_app = ANDROID_APP
        .get()
        .cloned()
        .expect("app not correctly initialized");
    winit::event_loop::EventLoop::builder()
        .with_android_app(android_app)
        .build()
        .expect("graphics initialization failed")
}

pub(crate) fn init_logging(level: log::Level) {
    let config = android_logger::Config::default().with_max_level(log::LevelFilter::Trace); // allow all levels at compile time
    android_logger::init_once(config);
    log::set_max_level(level.to_level_filter());
}
