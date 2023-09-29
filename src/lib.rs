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
pub use game::ExitFlag;
pub use game::Game;
pub use game::GameCommand;
pub use game::GameData;
pub use game::GameInitialisationFailure;
use wgpu::util::DeviceExt;
pub mod input {
    pub use crate::game::input::*;
}

fn next_multiple_of(
    value: wgpu::BufferAddress,
    multiple: wgpu::BufferAddress,
) -> wgpu::BufferAddress {
    match value % multiple {
        0 => value,
        r => value + (multiple - r),
    }
}

/// Some operations care about alignment in such a way that it is often easier to simply round all buffer sizes up to the nearest
/// multiple of some power of two. This constant gives that power of two, and the corresponding [`LfDeviceExt::create_buffer_padded`],
/// [`LfDeviceExt::new_buffer_init_padded`] and [`LfQueueExt::write_buffer_padded`] all extend their data lengths to the nearest
/// multiple of this constant.
pub const BUFFER_PADDING: wgpu::BufferAddress = 256;

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

    pub trait SealedQueue {}
    impl SealedQueue for wgpu::Queue {}

    pub trait SealedBindGroupLayoutEntry {}
    impl SealedBindGroupLayoutEntry for wgpu::BindGroupLayoutEntry {}

    // We even want to extend our own objects
    pub trait SealedGame {}
    impl<T: crate::Game> SealedGame for T {}
}

pub struct PaddedBufferInitDescriptor<'a> {
    /// Debug label of a buffer. This will show up in graphics debuggers for easy identification.
    pub label: wgpu::Label<'a>,
    /// Contents of a buffer on creation. Will be extended to the next pad interval.
    pub contents: Vec<u8>,
    /// Usages of a buffer. If the buffer is used in any way that isn't specified here, the operation
    /// will panic.
    pub usage: wgpu::BufferUsages,
}

/// Extensions to [`wgpu::Device`].
pub trait LfDeviceExt: sealed::SealedDevice {
    fn create_buffer_padded(&self, desc: wgpu::BufferDescriptor) -> wgpu::Buffer;
    fn create_buffer_init_padded(&self, desc: PaddedBufferInitDescriptor) -> wgpu::Buffer;

    fn create_fragment_only_render_bundle_encoder(
        &self,
        desc: &FragmentOnlyRenderBundleEncoderDescriptor,
    ) -> FragmentOnlyRenderBundleEncoder;

    fn create_fragment_only_render_pipeline(
        &self,
        desc: &FragmentOnlyRenderPipelineDescriptor,
    ) -> FragmentOnlyRenderPipeline;

    /// Creates a module, either with `create_shader_module` on debug or wasm, or `create_shader_module_unchecked` on release.
    ///
    /// Safety requirements carry from `create_shader_module_unchecked`.
    unsafe fn create_shader_module_unchecked_on_release(
        &self,
        desc: wgpu::ShaderModuleDescriptor,
    ) -> wgpu::ShaderModule;
}

impl LfDeviceExt for wgpu::Device {
    fn create_buffer_padded(&self, mut desc: wgpu::BufferDescriptor) -> wgpu::Buffer {
        desc.size = next_multiple_of(desc.size, BUFFER_PADDING);

        self.create_buffer(&desc)
    }

    fn create_buffer_init_padded(&self, mut desc: PaddedBufferInitDescriptor) -> wgpu::Buffer {
        let new_len = next_multiple_of(desc.contents.len() as wgpu::BufferAddress, BUFFER_PADDING);
        desc.contents.resize(new_len as usize, 0u8);

        self.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: desc.label,
            contents: &desc.contents,
            usage: desc.usage,
        })
    }

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

    unsafe fn create_shader_module_unchecked_on_release(
        &self,
        desc: wgpu::ShaderModuleDescriptor,
    ) -> wgpu::ShaderModule {
        #[cfg(any(target_arch = "wasm", debug_assertions))]
        {
            self.create_shader_module(desc)
        }
        #[cfg(not(any(target_arch = "wasm", debug_assertions)))]
        {
            self.create_shader_module_unchecked(desc)
        }
    }
}

/// Extensions to [`wgpu::CommandEncoder`].
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

/// Extensions to [`wgpu::Limits`].
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

/// Extensions to [`wgpu::Queue`].
pub trait LfQueueExt: sealed::SealedQueue {
    /// Writes the given data to the given buffer using [`wgpu::Queue::write_buffer`],
    /// but pads the data to the nearest multiple of the alignment required for buffer writing.
    ///
    /// # Panics
    ///
    /// Panics if the padded data would overrun the given buffer.
    fn write_buffer_padded(
        &self,
        buffer: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        data: Vec<u8>,
    );
}

impl LfQueueExt for wgpu::Queue {
    fn write_buffer_padded(
        &self,
        buffer: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        mut data: Vec<u8>,
    ) {
        const PAD_ALIGNMENT: usize = BUFFER_PADDING as usize;
        let len = data.len();
        let target_size = match len % PAD_ALIGNMENT {
            0 => len,
            r => len + (PAD_ALIGNMENT - r),
        };
        data.resize(target_size, 0u8);

        self.write_buffer(buffer, offset, &data)
    }
}

/// Extensions to [`wgpu::Buffer`].
pub trait LfBufferExt: sealed::SealedBuffer {
    /// Blocks and reads the entire buffer, giving the bytes contained. Allocates the temporary staging buffer for
    /// this operation. Panics on error, or if the buffer was not created with `wgpu::BufferUsages::COPY_SRC`.
    ///
    /// Just use `wgpu::Queue::write_buffer` if you want to write.
    fn debug_read_blocking(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u8>;
}

impl LfBufferExt for wgpu::Buffer {
    fn debug_read_blocking(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u8> {
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

/// Extensions to [`wgpu::BindGroupLayoutEntry`].
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

/// Extensions to an implemented game object.
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
