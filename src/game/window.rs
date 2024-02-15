use std::sync::Arc;

use wgpu::Device;
use winit::{event_loop::EventLoopWindowTarget, window::WindowBuilder};

pub struct GameWindow {
    window: Arc<winit::window::Window>,

    #[cfg(target_arch = "wasm32")]
    canvas: web_sys::HtmlCanvasElement,
}

impl GameWindow {
    pub(super) fn new<T: super::Game>(window_target: &EventLoopWindowTarget<()>) -> Self {
        let builder = WindowBuilder::new().with_title(T::title());
        #[cfg(target_arch = "wasm32")]
        let canvas = crate::wasm::get_canvas();
        #[cfg(target_arch = "wasm32")]
        let builder = {
            use winit::platform::web::WindowBuilderExtWebSys;
            builder
                .with_prevent_default(true)
                .with_focusable(true)
                .with_canvas(Some(canvas.clone()))
        };
        let window = builder.build(window_target).unwrap();
        let window = Arc::new(window);

        Self {
            window,
            #[cfg(target_arch = "wasm32")]
            canvas,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn canvas(&self) -> web_sys::HtmlCanvasElement {
        self.canvas.clone()
    }

    pub(crate) fn create_surface(
        &self,
        instance: &wgpu::Instance,
    ) -> Result<wgpu::Surface<'static>, wgpu::CreateSurfaceError> {
        instance.create_surface(Arc::clone(&self.window))
    }
}

impl std::ops::Deref for GameWindow {
    type Target = winit::window::Window;

    fn deref(&self) -> &Self::Target {
        &self.window
    }
}

/// Something that needs remaking/resizing whenever the game window is resized
pub trait WindowSizeDependent {
    fn on_window_resize(&mut self, device: &Device, new_size: winit::dpi::PhysicalSize<u32>);
}
