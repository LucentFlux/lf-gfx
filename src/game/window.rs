use std::sync::Arc;

use wgpu::Device;
use winit::{event_loop::ActiveEventLoop, window::WindowAttributes};

pub struct GameWindow {
    window: Arc<winit::window::Window>,

    #[cfg(target_arch = "wasm32")]
    canvas: web_sys::HtmlCanvasElement,
}

impl GameWindow {
    pub(super) fn new<T: super::Game>(window_target: &ActiveEventLoop) -> Self {
        let mut attributes = WindowAttributes::default();
        attributes.title = T::title().into();

        #[cfg(target_arch = "wasm32")]
        let canvas = {
            use winit::platform::web::WindowAttributesExtWebSys;
            let canvas = crate::wasm::get_canvas();
            attributes = attributes.with_canvas(Some(canvas.clone()));
            canvas
        };

        let window = window_target
            .create_window(attributes)
            .expect("Failed to create window");
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
