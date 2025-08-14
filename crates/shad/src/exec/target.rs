use std::sync::Arc;
use wgpu::{Surface, SurfaceConfiguration, SurfaceTexture, Texture, TextureView};
use winit::window::Window;

#[derive(Debug)]
pub(crate) struct Target {
    pub(crate) inner: TargetSpecialized,
    pub(crate) config: TargetConfig,
    pub(crate) depth_buffer: TextureView,
}

#[derive(Debug)]
pub(crate) enum TargetSpecialized {
    Window(WindowTarget),
    Texture(TextureTarget),
}

#[derive(Debug)]
pub(crate) struct TextureTarget {
    pub(crate) texture: Texture,
    pub(crate) view: TextureView,
}

// coverage: off (window cannot be tested)

#[derive(Debug)]
pub(crate) struct WindowTarget {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: Surface<'static>,
    pub(crate) surface_config: SurfaceConfiguration,
}

impl WindowTarget {
    pub(crate) fn create_surface_texture(&self) -> SurfaceTexture {
        self.surface
            .get_current_texture()
            .expect("internal error: cannot retrieve surface texture")
    }
}

// coverage: on

#[derive(Debug)]
pub(crate) struct TargetConfig {
    pub(crate) size: (u32, u32),
}
