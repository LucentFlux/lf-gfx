use core::num::NonZeroU32;
use std::borrow::Cow;

use wgpu::util::{DeviceExt, RenderEncoder};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
pub struct FullscreenVertex {
    pub position: [f32; 4],
    pub uv: [f32; 2],
}

pub struct FragmentOnlyRenderPipelineDescriptor<'a> {
    pub label: wgpu::Label<'a>,
    pub layout: Option<&'a wgpu::PipelineLayout>,
    pub multisample: wgpu::MultisampleState,
    pub fragment: wgpu::FragmentState<'a>,
    pub multiview: Option<NonZeroU32>,
}

pub struct FragmentOnlyRenderPipeline {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
}

impl AsRef<wgpu::RenderPipeline> for FragmentOnlyRenderPipeline {
    fn as_ref(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}

impl FragmentOnlyRenderPipeline {
    pub(crate) fn new(device: &wgpu::Device, desc: &FragmentOnlyRenderPipelineDescriptor) -> Self {
        let fullscreen_vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fullscreen triangle vertex shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::from(include_str!("shaders/fullscreen.wgsl"))),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: desc.label.as_deref(),
            layout: desc.layout.clone(),
            vertex: wgpu::VertexState {
                module: &fullscreen_vertex_shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<FullscreenVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: desc.multisample.clone(),
            fragment: Some(desc.fragment.clone()),
            multiview: desc.multiview.clone(),
        });

        let fullscreen_vertices = [
            FullscreenVertex {
                position: [-2.0, -1.0, 0.5, 1.0],
                uv: [-2.0, -1.0],
            },
            FullscreenVertex {
                position: [2.0, -1.0, 0.5, 1.0],
                uv: [2.0, -1.0],
            },
            FullscreenVertex {
                position: [0.0, 2.0, 0.5, 1.0],
                uv: [0.0, 2.0],
            },
        ];
        let fullscreen_vertices_bytes: &[u8] = bytemuck::cast_slice(&fullscreen_vertices);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fullscreen triangle"),
            contents: fullscreen_vertices_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            vertex_buffer,
        }
    }

    pub fn get_bind_group_layout(&self, index: u32) -> wgpu::BindGroupLayout {
        self.pipeline.get_bind_group_layout(index)
    }
}

fn set_pipeline<'a>(
    encoder: &mut impl RenderEncoder<'a>,
    pipeline: &'a FragmentOnlyRenderPipeline,
) {
    encoder.set_pipeline(&pipeline.pipeline);
    encoder.set_vertex_buffer(0, pipeline.vertex_buffer.slice(..));
}

fn draw<'a>(encoder: &mut impl RenderEncoder<'a>) {
    encoder.draw(0..3, 0..1)
}

fn set_push_constants<'a>(encoder: &mut impl RenderEncoder<'a>, offset: u32, data: &[u8]) {
    encoder.set_push_constants(wgpu::ShaderStages::FRAGMENT, offset, data)
}

pub struct FragmentOnlyRenderBundleEncoderDescriptor<'a> {
    pub label: wgpu::Label<'a>,
    pub color_formats: &'a [Option<wgpu::TextureFormat>],
    pub sample_count: u32,
    pub multiview: Option<NonZeroU32>,
}

pub struct FragmentOnlyRenderBundleEncoder<'a> {
    encoder: wgpu::RenderBundleEncoder<'a>,
}

impl<'a> FragmentOnlyRenderBundleEncoder<'a> {
    pub(crate) fn new(
        device: &'a wgpu::Device,
        desc: &FragmentOnlyRenderBundleEncoderDescriptor,
    ) -> Self {
        let encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
            label: desc.label.as_deref(),
            color_formats: desc.color_formats,
            depth_stencil: None,
            sample_count: desc.sample_count,
            multiview: desc.multiview,
        });

        Self { encoder }
    }

    // Mostly pass-through
    pub fn finish(self, desc: &wgpu::RenderBundleDescriptor<'_>) -> FragmentOnlyRenderBundle {
        let render_bundle = self.encoder.finish(desc);

        FragmentOnlyRenderBundle { render_bundle }
    }
    pub fn set_pipeline(&mut self, pipeline: &'a FragmentOnlyRenderPipeline) {
        set_pipeline(&mut self.encoder, pipeline)
    }
    pub fn draw(&mut self) {
        draw(&mut self.encoder)
    }
    pub fn set_push_constants(&mut self, offset: u32, data: &[u8]) {
        set_push_constants(&mut self.encoder, offset, data)
    }
}

pub struct FragmentOnlyRenderBundle {
    render_bundle: wgpu::RenderBundle,
}

pub struct FragmentOnlyRenderPassStencilAttachment<'tex> {
    pub view: &'tex wgpu::TextureView,
    pub stencil_ops: Option<wgpu::Operations<u32>>,
}

pub struct FragmentOnlyRenderPassDescriptor<'tex, 'desc> {
    pub label: wgpu::Label<'desc>,
    pub color_attachments: &'desc [Option<wgpu::RenderPassColorAttachment<'tex>>],
    pub stencil_attachment: Option<FragmentOnlyRenderPassStencilAttachment<'tex>>,
}

#[derive(Debug)]
pub struct FragmentOnlyRenderPass<'a> {
    renderpass: wgpu::RenderPass<'a>,
}

impl<'a> FragmentOnlyRenderPass<'a> {
    pub(crate) fn new(
        command_encoder: &'a mut wgpu::CommandEncoder,
        desc: &FragmentOnlyRenderPassDescriptor<'a, '_>,
    ) -> Self {
        let desc = wgpu::RenderPassDescriptor {
            label: desc.label,
            color_attachments: desc.color_attachments,
            depth_stencil_attachment: desc.stencil_attachment.as_ref().map(|attachment| {
                wgpu::RenderPassDepthStencilAttachment {
                    view: attachment.view,
                    depth_ops: None,
                    stencil_ops: attachment.stencil_ops,
                }
            }),
        };

        let renderpass = command_encoder.begin_render_pass(&desc);

        Self { renderpass }
    }

    // These methods have changed

    pub fn set_pipeline(&mut self, pipeline: &'a FragmentOnlyRenderPipeline) {
        set_pipeline(&mut self.renderpass, pipeline)
    }
    pub fn draw(&mut self) {
        draw(&mut self.renderpass)
    }
    pub fn set_push_constants(&mut self, offset: u32, data: &[u8]) {
        set_push_constants(&mut self.renderpass, offset, data)
    }
    pub fn execute_bundles<I: IntoIterator<Item = &'a FragmentOnlyRenderBundle>>(
        &mut self,
        render_bundles: I,
    ) {
        let render_bundles: Vec<&wgpu::RenderBundle> = render_bundles
            .into_iter()
            .map(|bundle| &bundle.render_bundle)
            .collect();

        self.renderpass.execute_bundles(render_bundles)
    }

    // We just pass most things through

    /// See wgpu::RenderPass
    pub fn set_bind_group(
        &mut self,
        index: u32,
        bind_group: &'a wgpu::BindGroup,
        offsets: &[wgpu::DynamicOffset],
    ) {
        self.renderpass.set_bind_group(index, bind_group, offsets)
    }
    pub fn set_blend_constant(&mut self, color: wgpu::Color) {
        self.renderpass.set_blend_constant(color)
    }
    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.renderpass.set_scissor_rect(x, y, width, height)
    }
    pub fn set_stencil_reference(&mut self, reference: u32) {
        self.renderpass.set_stencil_reference(reference)
    }
    pub fn insert_debug_marker(&mut self, label: &str) {
        self.renderpass.insert_debug_marker(label)
    }
    pub fn push_debug_group(&mut self, label: &str) {
        self.renderpass.push_debug_group(label)
    }
    pub fn pop_debug_group(&mut self) {
        self.renderpass.pop_debug_group()
    }
    pub fn write_timestamp(&mut self, query_set: &wgpu::QuerySet, query_index: u32) {
        self.renderpass.write_timestamp(query_set, query_index)
    }
    pub fn begin_pipeline_statistics_query(
        &mut self,
        query_set: &wgpu::QuerySet,
        query_index: u32,
    ) {
        self.renderpass
            .begin_pipeline_statistics_query(query_set, query_index)
    }
    pub fn end_pipeline_statistics_query(&mut self) {
        self.renderpass.end_pipeline_statistics_query()
    }
}
