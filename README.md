# LucentFlux Graphics Library

This crate provides a collection of common utilities when writing wgpu programs. This library is made public since the utilities provided may be useful, but is not intended to be a general-purpose library and features *will* be added and removed as and when we need them.

# Features

## Fragment-only fullscreen-quad shader pipelines

Reduced boilerplate for fragment shaders run on fullscreen quads:

```rust no_run
use lf_gfx::{LfDeviceExt, LfCommandEncoderExt};
# let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
# let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();
# let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

// Standard pipeline creation pattern, but ending with `create_fragment_only_render_pipeline`:
let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { 
    label: Some("layout"), 
    bind_group_layouts: &[/* .. */], 
    push_constant_ranges:  &[/* .. */] 
});
let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("module"),
    source: wgpu::ShaderSource::Wgsl(
        /* .. */
#       "".into()
    ),
});
let pipeline = device.create_fragment_only_render_pipeline(&lf_gfx::FragmentOnlyRenderPipelineDescriptor {
    label: Some("pipeline"),
    layout: Some(&layout),
    multisample: wgpu::MultisampleState::default(),
    fragment: wgpu::FragmentState { 
        module: &module, 
        entry_point: "main", 
        targets: &[/* .. */] 
    },
    multiview: None,
});

let mut cmd = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
    label: Some("command encoder"),
});
let mut rp = cmd.begin_fragment_only_render_pass(&lf_gfx::FragmentOnlyRenderPassDescriptor {
    label: Some("renderpass"),
    color_attachments: &[/* .. */],
    stencil_attachment: None,
    timestamp_writes: None,
});
// No need to bind vertex buffers or even a unit vertex shader
rp.set_pipeline(&pipeline);
rp.draw();
```

## Local Storage

```rust 
use lf_gfx::local_storage;

local_storage::store("my_key", "a value");

// Persists between runs
let stored = local_storage::load("my_key");
assert_eq!(stored, Some("a value".to_owned()));
```

## Game API

```rust no_run
struct MyGameCfg { /* .. */ }
#[derive(serde::Serialize, serde::Deserialize)]
enum MyGameLinearInputs { 
    Forward,
    Jump,
    /* .. */ 
}
#[derive(serde::Serialize, serde::Deserialize)]
enum MyGameVectorInputs { 
    Look,
    /* .. */ 
}

struct MyGame { /* .. */ }

impl lf_gfx::Game for MyGame {
    type InitData = MyGameCfg;
    type LinearInputType = MyGameLinearInputs;
    type VectorInputType = MyGameVectorInputs;

    fn title() -> &'static str {
        "My Game"
    }

    fn default_inputs(&self) -> lf_gfx::input::InputMap<MyGameLinearInputs, MyGameVectorInputs> {
        let mut inputs = lf_gfx::input::InputMap::empty();

        inputs.assign_linear(lf_gfx::input::KeyCode::KeyW, MyGameLinearInputs::Forward);
        inputs.assign_linear(lf_gfx::input::MouseInputType::ScrollUp, MyGameLinearInputs::Jump);

        inputs.assign_vector(lf_gfx::input::VectorInputType::MouseMove, MyGameVectorInputs::Look);

        return inputs;
    }

    fn init(data: &lf_gfx::GameData, init: Self::InitData) -> Self {
        Self { /* .. */ }
    }

    fn window_resize(&mut self, data: &lf_gfx::GameData, new_size: winit::dpi::PhysicalSize<u32>) {
        /* .. */
    }

    fn handle_linear_input(
        &mut self,
        data: &lf_gfx::GameData,
        input: &Self::LinearInputType,
        activation: lf_gfx::input::LinearInputActivation,
    ) {
        /* .. */
    }

    fn handle_vector_input(
        &mut self,
        data: &lf_gfx::GameData,
        input: &Self::VectorInputType,
        activation: lf_gfx::input::VectorInputActivation,
    ) {
        /* .. */
    }

    fn render_to(&mut self, data: &lf_gfx::GameData, view: wgpu::TextureView) {
        /* .. */
    }
}

fn main() {
    use lf_gfx::LfGameExt;
    MyGame::run(MyGameCfg { /* .. */ });
}
```