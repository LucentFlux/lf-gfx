mod adapter_query;
mod fragment_only;

pub use adapter_query::AdapterQuery;
pub use fragment_only::FragmentOnlyRenderBundleEncoder;
pub use fragment_only::FragmentOnlyRenderBundleEncoderDescriptor;
pub use fragment_only::FragmentOnlyRenderPass;
pub use fragment_only::FragmentOnlyRenderPassDescriptor;
pub use fragment_only::FragmentOnlyRenderPipeline;
pub use fragment_only::FragmentOnlyRenderPipelineDescriptor;

// Link in to existing objects
// We're only adding methods to specific wgpu objects
mod sealed {
    pub trait SealedDevice {}
    impl SealedDevice for wgpu::Device {}

    pub trait SealedInstance {}
    impl SealedInstance for wgpu::Instance {}

    pub trait SealedCommandEncoder {}
    impl SealedCommandEncoder for wgpu::CommandEncoder {}
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
    async fn request_powerful_adapter<'a>(
        &self,
        backends: wgpu::Backends,
        query: AdapterQuery<'a>,
    ) -> Option<wgpu::Adapter>;
}

#[async_trait::async_trait]
impl InstanceExt for wgpu::Instance {
    async fn request_powerful_adapter<'a>(
        &self,
        backends: wgpu::Backends,
        query: AdapterQuery<'a>,
    ) -> Option<wgpu::Adapter> {
        adapter_query::request_powerful_adapter(self, backends, query).await
    }
}
