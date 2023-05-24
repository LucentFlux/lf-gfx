use wgpu_async::AsyncDevice;

/// Something that needs remaking/resizing whenever the game window is resized
pub trait WindowSizeDependent {
    fn on_window_resize(&mut self, device: &AsyncDevice, new_size: winit::dpi::PhysicalSize<u32>);
}
