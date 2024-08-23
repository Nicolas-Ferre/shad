#![allow(missing_docs)]

use crate::app::App;
use log::Level;

mod app;
mod buffer;
mod mesh;
mod platform;
mod program;
mod shader;

#[macro_export]
macro_rules! main {
    () => {
        #[cfg(target_os = "android")]
        #[no_mangle]
        extern "C" fn android_main(app: android_activity::AndroidApp) {
            $crate::init_android(app);
            $crate::run()
        }

        // Unused main method to remove Clippy warning
        #[cfg(target_os = "android")]
        #[allow(dead_code)]
        fn main() {}

        #[cfg(not(target_os = "android"))]
        fn main() {
            $crate::run()
        }
    };
}

pub fn run() {
    platform::init_logging(Level::Info);
    let event_loop = platform::event_loop();
    let app = App::default();
    platform::run_event_loop(event_loop, app);
}

#[cfg(target_os = "android")]
pub fn init_android(app: android_activity::AndroidApp) {
    let _ = platform::ANDROID_APP.get_or_init(move || app);
}
