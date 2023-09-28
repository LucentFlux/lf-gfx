# LucentFlux Graphics Library

This crate provides a collection of common utilities when writing wgpu programs. This library is made public since the utilities provided may be useful, but is not intended to be a general-purpose library and features *will* be added and removed as and when we need them.

# Features

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

## Event-based wgpu & winit/canvas Game Boilerplate

## Feature Unions