# LucentFlux Graphics Library

This crate provides a collection of common utilities when writing wgpu programs. 

# Features

## More adapter querying

Query on an instance, but provide extended query parameters:

```rust
use lf_gfx::InstanceExt;

let instance = wgpu::Instance::new(wgpu::Backends::all());

let primary_adapter = instance.request_powerful_adapter(wgpu::Backends::all(), AdapterQuery::default());

let secondary_adapter = instance.request_powerful_adapter(wgpu::Backends::all(), AdapterQuery {
    physical_blacklist: &[&primary_adapter]
    ..Default::default()
});
```

## Fragment-only fullscreen-quad shader pipelines

Reduced boilerplate for fragment shaders run on fullscreen quads:

```rust
use lf_gfx::{DeviceExt, CommandEncoderExt};

let device, queue = ...;

let pipeline = device.create_fragment_only_render_pipeline(...);

let cmd = device.create_command_encoder(...);
let rp = cmd.begin_fragment_only_render_pass(...);
// No need to bind vertex buffers or even a unit vertex shader
rp.set_pipeline(pipeline);
rp.set_bind_group(...);
rp.draw();
```