//! On web, we want to be able to resize/recreate the surface locklessly. We do this here.

use std::sync::{atomic::AtomicU32, Arc};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ResizableSurfaceState {
    Active,
    ResizingQueued,
    Inactive,
}

impl ResizableSurfaceState {
    fn encode(self) -> u32 {
        match self {
            Self::Active => 0,
            Self::ResizingQueued => 1,
            Self::Inactive => 2,
        }
    }

    fn decode(value: u32) -> Self {
        match value {
            0 => Self::Active,
            1 => Self::ResizingQueued,
            2 => Self::Inactive,
            _ => unreachable!(),
        }
    }
}

pub(super) struct ResizableSurface {
    surface: wgpu::Surface,

    config: wgpu::SurfaceConfiguration,

    state: Arc<AtomicU32>,
    new_size: winit::dpi::PhysicalSize<u32>,
}

impl ResizableSurface {
    pub(super) fn new(
        surface: wgpu::Surface,
        device: &wgpu::Device,
        config: wgpu::SurfaceConfiguration,
    ) -> Self {
        surface.configure(&device, &config);
        Self {
            surface,
            config,
            state: Arc::new(AtomicU32::new(ResizableSurfaceState::Active.encode())),
            new_size: winit::dpi::PhysicalSize::new(0, 0),
        }
    }

    pub(super) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, queue: &wgpu::Queue) {
        self.state.store(
            ResizableSurfaceState::ResizingQueued.encode(),
            std::sync::atomic::Ordering::SeqCst,
        );
        self.new_size = new_size;

        let state_clone = Arc::clone(&self.state);
        queue.on_submitted_work_done(move || {
            let _ = state_clone.compare_exchange(
                ResizableSurfaceState::ResizingQueued.encode(),
                ResizableSurfaceState::Inactive.encode(),
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            );
        })
    }

    pub(super) fn get(&mut self, device: &wgpu::Device) -> Option<&wgpu::Surface> {
        let state = self.state.load(std::sync::atomic::Ordering::SeqCst);
        let state = ResizableSurfaceState::decode(state);

        // Still waiting on resizing
        match state {
            ResizableSurfaceState::ResizingQueued => {
                return None;
            }
            ResizableSurfaceState::Inactive => {
                self.config.width = self.new_size.width;
                self.config.height = self.new_size.height;
                self.surface.configure(device, &self.config);

                let res = self.state.compare_exchange(
                    ResizableSurfaceState::Inactive.encode(),
                    ResizableSurfaceState::Active.encode(),
                    std::sync::atomic::Ordering::SeqCst,
                    std::sync::atomic::Ordering::SeqCst,
                );

                if res.is_ok() {
                    Some(&self.surface)
                } else {
                    None
                }
            }
            ResizableSurfaceState::Active => Some(&self.surface),
        }
    }
}
