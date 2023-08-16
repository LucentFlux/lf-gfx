mod fragment_only;
mod game;
mod limits;

pub use fragment_only::FragmentOnlyRenderBundleEncoder;
pub use fragment_only::FragmentOnlyRenderBundleEncoderDescriptor;
pub use fragment_only::FragmentOnlyRenderPass;
pub use fragment_only::FragmentOnlyRenderPassDescriptor;
pub use fragment_only::FragmentOnlyRenderPipeline;
pub use fragment_only::FragmentOnlyRenderPipelineDescriptor;
pub use game::window_size::WindowSizeDependent;
pub use game::Game;
pub use game::GameCommand;
pub use game::GameData;
pub use game::GameInitialisationFailure;
pub mod input {
    pub use crate::game::input::*;
}

// Link in to existing objects
// We're only adding methods to specific wgpu objects
mod sealed {
    pub trait SealedDevice {}
    impl SealedDevice for wgpu::Device {}

    pub trait SealedInstance {}
    impl SealedInstance for wgpu::Instance {}

    pub trait SealedCommandEncoder {}
    impl SealedCommandEncoder for wgpu::CommandEncoder {}

    pub trait SealedLimits {}
    impl SealedLimits for wgpu::Limits {}

    pub trait SealedBuffer {}
    impl SealedBuffer for wgpu::Buffer {}

    pub trait SealedBindGroupLayoutEntry {}
    impl SealedBindGroupLayoutEntry for wgpu::BindGroupLayoutEntry {}

    // We even want to extend our own objects
    pub trait SealedGame {}
    impl<T: crate::Game> SealedGame for T {}
}

pub trait LfDeviceExt: sealed::SealedDevice {
    fn create_fragment_only_render_bundle_encoder(
        &self,
        desc: &FragmentOnlyRenderBundleEncoderDescriptor,
    ) -> FragmentOnlyRenderBundleEncoder;

    fn create_fragment_only_render_pipeline(
        &self,
        desc: &FragmentOnlyRenderPipelineDescriptor,
    ) -> FragmentOnlyRenderPipeline;
}

impl LfDeviceExt for wgpu::Device {
    fn create_fragment_only_render_bundle_encoder(
        &self,
        desc: &FragmentOnlyRenderBundleEncoderDescriptor,
    ) -> FragmentOnlyRenderBundleEncoder {
        FragmentOnlyRenderBundleEncoder::new(self, desc)
    }

    fn create_fragment_only_render_pipeline(
        &self,
        desc: &FragmentOnlyRenderPipelineDescriptor,
    ) -> FragmentOnlyRenderPipeline {
        FragmentOnlyRenderPipeline::new(self, desc)
    }
}

pub trait LfCommandEncoderExt: sealed::SealedCommandEncoder {
    fn begin_fragment_only_render_pass<'pass>(
        &'pass mut self,
        desc: &FragmentOnlyRenderPassDescriptor<'pass, '_>,
    ) -> FragmentOnlyRenderPass<'pass>;
}

impl LfCommandEncoderExt for wgpu::CommandEncoder {
    fn begin_fragment_only_render_pass<'pass>(
        &'pass mut self,
        desc: &FragmentOnlyRenderPassDescriptor<'pass, '_>,
    ) -> FragmentOnlyRenderPass<'pass> {
        FragmentOnlyRenderPass::new(self, desc)
    }
}

pub trait LfLimitsExt: sealed::SealedLimits {
    /// Gets the set of limits supported both by this and the other limits.
    fn intersection<'a>(&self, other: &wgpu::Limits) -> wgpu::Limits;
    /// Gets the set of limits supported by either this ot the other limits.
    fn union<'a>(&self, other: &wgpu::Limits) -> wgpu::Limits;
}

impl LfLimitsExt for wgpu::Limits {
    /// Gets the set of limits supported both by this and the other limits.
    fn intersection<'a>(&self, other: &wgpu::Limits) -> wgpu::Limits {
        crate::limits::limits_intersection(self, other)
    }
    /// Gets the set of limits supported by either this ot the other limits.
    fn union<'a>(&self, other: &wgpu::Limits) -> wgpu::Limits {
        crate::limits::limits_union(self, other)
    }
}

pub trait LfBufferExt: sealed::SealedBuffer {
    /// Blocks and reads the entire buffer, giving the bytes contained. Allocates the temporary staging buffer for
    /// this operation. Panics on error, or if the buffer was not created with `wgpu::BufferUsages::COPY_SRC`.
    ///
    /// Just use `wgpu::Queue::write_buffer` if you want to write.
    ///
    /// # Panics
    ///
    /// Panics if this is a release build, since this method should only be used while debugging.
    fn debug_read_blocking(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u8>;
}

impl LfBufferExt for wgpu::Buffer {
    fn debug_read_blocking(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u8> {
        #[cfg(not(debug_assertions))]
        panic!("debug_read_blocking should never be used in release contexts");

        assert!(self.usage().contains(wgpu::BufferUsages::COPY_SRC));

        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("debug-read-staging"),
            size: self.size(),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut cmd = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("debug-read-cmd-encoder"),
        });
        cmd.copy_buffer_to_buffer(self, 0, &staging, 0, self.size());

        queue.submit(vec![cmd.finish()]);

        let (sender, receiver) = std::sync::mpsc::channel();
        staging.slice(..).map_async(wgpu::MapMode::Read, move |e| {
            sender.send(e).expect("failed to send result of map");
        });

        device.poll(wgpu::Maintain::Wait);

        receiver
            .recv()
            .expect("failed to get result of map")
            .expect("failed to read buffer");

        let slice = staging.slice(..).get_mapped_range();
        slice.to_vec()
    }
}

pub trait LfBindGroupLayoutEntryExt: sealed::SealedBindGroupLayoutEntry {
    // Some common bindings as constructors
    fn read_only_compute_storage(binding: u32) -> Self;
    fn mutable_compute_storage(binding: u32) -> Self;
}

impl LfBindGroupLayoutEntryExt for wgpu::BindGroupLayoutEntry {
    fn read_only_compute_storage(binding: u32) -> Self {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn mutable_compute_storage(binding: u32) -> Self {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
}

pub trait LfGameExt: sealed::SealedGame {
    type InitData;

    /// Runs the game.
    fn run(init: Self::InitData);
}

impl<T: Game + 'static> LfGameExt for T {
    type InitData = T::InitData;

    /// Runs the game.
    fn run(init: T::InitData) {
        game::GameState::<T>::run(init);
    }
}
