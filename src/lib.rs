mod adapter_query;
mod fragment_only;
mod game;
mod limits;

pub use adapter_query::AdapterQuery;
pub use fragment_only::FragmentOnlyRenderBundleEncoder;
pub use fragment_only::FragmentOnlyRenderBundleEncoderDescriptor;
pub use fragment_only::FragmentOnlyRenderPass;
pub use fragment_only::FragmentOnlyRenderPassDescriptor;
pub use fragment_only::FragmentOnlyRenderPipeline;
pub use fragment_only::FragmentOnlyRenderPipelineDescriptor;
pub use game::window_size::WindowSizeDependent;
pub use game::Game;
pub use game::GameCommand;
pub use game::GameInitData;
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

    // We even want to extend our own objects
    pub trait SealedGame {}
    impl<T: crate::Game> SealedGame for T {}
}

pub trait DeviceExt: sealed::SealedDevice {
    fn create_fragment_only_render_bundle_encoder(
        &self,
        desc: &FragmentOnlyRenderBundleEncoderDescriptor,
    ) -> FragmentOnlyRenderBundleEncoder;

    fn create_fragment_only_render_pipeline(
        &self,
        desc: &FragmentOnlyRenderPipelineDescriptor,
    ) -> FragmentOnlyRenderPipeline;
}

impl DeviceExt for wgpu::Device {
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

pub trait CommandEncoderExt: sealed::SealedCommandEncoder {
    fn begin_fragment_only_render_pass<'pass>(
        &'pass mut self,
        desc: &FragmentOnlyRenderPassDescriptor<'pass, '_>,
    ) -> FragmentOnlyRenderPass<'pass>;
}

impl CommandEncoderExt for wgpu::CommandEncoder {
    fn begin_fragment_only_render_pass<'pass>(
        &'pass mut self,
        desc: &FragmentOnlyRenderPassDescriptor<'pass, '_>,
    ) -> FragmentOnlyRenderPass<'pass> {
        FragmentOnlyRenderPass::new(self, desc)
    }
}

#[async_trait::async_trait]
pub trait InstanceExt: sealed::SealedInstance {
    /// Gets (some notion of) the most powerful adapter available, given the constraints provided.
    fn request_powerful_adapter<'a>(
        &self,
        backends: wgpu::Backends,
        query: AdapterQuery<'a>,
    ) -> Option<wgpu::Adapter>;
}

#[async_trait::async_trait]
impl InstanceExt for wgpu::Instance {
    fn request_powerful_adapter<'a>(
        &self,
        backends: wgpu::Backends,
        query: AdapterQuery<'a>,
    ) -> Option<wgpu::Adapter> {
        adapter_query::request_powerful_adapter(self, backends, query)
    }
}

#[async_trait::async_trait]
pub trait LimitsExt: sealed::SealedLimits {
    /// Gets the set of limits supported both by this and the other limits.
    fn intersection<'a>(&self, other: &wgpu::Limits) -> wgpu::Limits;
    /// Gets the set of limits supported by either this ot the other limits.
    fn union<'a>(&self, other: &wgpu::Limits) -> wgpu::Limits;
}

#[async_trait::async_trait]
impl LimitsExt for wgpu::Limits {
    /// Gets the set of limits supported both by this and the other limits.
    fn intersection<'a>(&self, other: &wgpu::Limits) -> wgpu::Limits {
        crate::limits::limits_intersection(self, other)
    }
    /// Gets the set of limits supported by either this ot the other limits.
    fn union<'a>(&self, other: &wgpu::Limits) -> wgpu::Limits {
        crate::limits::limits_union(self, other)
    }
}

pub trait BufferExt: sealed::SealedBuffer {
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

impl BufferExt for wgpu::Buffer {
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

#[async_trait::async_trait]
pub trait GameExt: sealed::SealedGame {
    /// Runs the game.
    fn run();
}

#[async_trait::async_trait]
impl<T: Game + 'static> GameExt for T {
    /// Runs the game.
    fn run() {
        game::GameState::<T>::run();
    }
}
