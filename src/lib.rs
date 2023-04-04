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
