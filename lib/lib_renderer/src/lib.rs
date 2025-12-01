use std::mem::{offset_of, transmute};

use image::EncodableLayout;
use lib_app::AppContext;

mod dyn_quad_buffer;
mod quad_buffer;
pub use dyn_quad_buffer::*;
pub use quad_buffer::*;

use lib_gpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType,
    BufferDescriptor, BufferInitDescriptor, BufferUsages, Color, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, Device, DeviceExt, Extent3d, FilterMode, FragmentState, FrontFace,
    IndexFormat, LoadOp, MultisampleState, Operations, Origin3d, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, SamplerDescriptor, ShaderStages, StoreOp, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode, include_wgsl,
};
use lib_math::{
    f32::{Vec2f, Vec4f},
    vec2,
};

const PIXELS_PER_UNIT: f32 = 16.0;
const ASPECT: f32 = 16.0 / 9.0;
const MAX_ORTHO_SIZE: f32 = 18.0;

#[derive(Debug)]
pub struct Renderer {
    vertex_buf: Buffer,
    index_buf: Buffer,
    render_texture_view: TextureView,
    quad_uniform_buf: Buffer,
    quad_bind_group: BindGroup,
    quad_pipeline: RenderPipeline,
    upscale_uniform_buf: Buffer,
    upscale_bind_group: BindGroup,
    upscale_pipeline: RenderPipeline,
}

#[derive(Debug)]
pub struct RendererFrame<'a> {
    renderer: &'a Renderer,
    output: &'a TextureView,
    ctx: RendererContext<'a>,
    ops: Operations<Color>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub center: Vec2f,
    pub sprite: Sprite,
    pub layer: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    pub center: Vec2f,
    pub extents: Vec2f,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub center: Vec2f,
    pub ortho_size: f32,
    pub clear_color: Vec4f,
}

#[derive(Debug, Clone, Copy)]
pub struct RendererContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub surface_format: TextureFormat,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct QuadUniform {
    cam_center: Vec2f,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct UpscaleUniform {
    src_extents: Vec2f,
    src_offset: Vec2f,
    dst_extents: Vec2f,
}

impl Renderer {
    pub fn new(ctx: RendererContext<'_>) -> Self {
        let vertex_buf = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("lib_renderer vertex buffer"),
            contents: unsafe {
                transmute::<&[Vec2f; 4], &[u8; 32]>(&[
                    vec2!(-1.0, -1.0),
                    vec2!(1.0, -1.0),
                    vec2!(1.0, 1.0),
                    vec2!(-1.0, 1.0),
                ])
            },
            usage: BufferUsages::VERTEX,
        });

        let index_buf = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("lib_renderer index buffer"),
            contents: unsafe { transmute::<&[u16; 6], &[u8; 12]>(&[0, 1, 2, 2, 3, 0]) },
            usage: BufferUsages::INDEX,
        });

        let render_texture = ctx.device.create_texture(&TextureDescriptor {
            label: Some("lib_renderer render texture"),
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            size: Extent3d {
                width: (PIXELS_PER_UNIT * MAX_ORTHO_SIZE * 2.0 * ASPECT) as u32,
                height: (PIXELS_PER_UNIT * MAX_ORTHO_SIZE * 2.0) as u32,
                depth_or_array_layers: 1,
            },
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let render_texture_view = render_texture.create_view(&TextureViewDescriptor::default());

        let quad_uniform_buf = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("lib_renderer quad buffer"),
            size: size_of::<QuadUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sprites = {
            let image = {
                let image = image::open(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../assets/textures/sprites.png"
                ))
                .expect("Failed to open lib_renderer sprites texture");

                image.to_rgba8()
            };

            let texture = ctx.device.create_texture(&TextureDescriptor {
                label: Some("lib_renderer sprites texture"),
                size: Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            });

            ctx.queue.write_texture(
                TexelCopyTextureInfo {
                    texture: &texture,
                    aspect: TextureAspect::All,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                },
                image.as_bytes(),
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(image.width() * 4),
                    rows_per_image: Some(image.height()),
                },
                texture.size(),
            );

            texture
        };

        let sampler = ctx.device.create_sampler(&SamplerDescriptor {
            label: Some("lib_renderer sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            anisotropy_clamp: 1,
            border_color: None,
            compare: None,
            lod_max_clamp: 1.0,
            lod_min_clamp: 1.0,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
        });

        let quad_shader = ctx
            .device
            .create_shader_module(include_wgsl!("quad_shader.wgsl"));

        let quad_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("lib_renderer quad bind group layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                            visibility: ShaderStages::VERTEX,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: false },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                            count: None,
                            visibility: ShaderStages::FRAGMENT,
                        },
                    ],
                });

        let quad_bind_group = ctx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("lib_renderer quad bind group"),
            layout: &quad_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: quad_uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &sprites.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let quad_pipeline = ctx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("lib_renderer quad pipeline"),
                cache: None,
                depth_stencil: None,
                layout: Some(
                    &ctx.device
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: Some("lib_renderer quad pipeline layout"),
                            bind_group_layouts: &[&quad_bind_group_layout],
                            push_constant_ranges: &[],
                        }),
                ),
                multiview: None,
                primitive: PrimitiveState {
                    front_face: FrontFace::Ccw,
                    conservative: false,
                    cull_mode: None,
                    polygon_mode: PolygonMode::Fill,
                    strip_index_format: None,
                    topology: PrimitiveTopology::TriangleList,
                    unclipped_depth: false,
                },
                vertex: VertexState {
                    module: &quad_shader,
                    entry_point: None,
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[VERTEX_BUFFER_LAYOUT, QUAD_BUFFER_LAYOUT],
                },
                fragment: Some(FragmentState {
                    module: &quad_shader,
                    targets: &[Some(ColorTargetState {
                        blend: Some(BlendState::ALPHA_BLENDING),
                        format: TextureFormat::Rgba8Unorm,
                        write_mask: ColorWrites::all(),
                    })],
                    entry_point: None,
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                multisample: MultisampleState::default(),
            });

        let upscale_shader = ctx
            .device
            .create_shader_module(include_wgsl!("upscale_shader.wgsl"));

        let upscale_uniform_buf = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("lib_renderer upscale uniform buffer"),
            mapped_at_creation: false,
            size: size_of::<UpscaleUniform>() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let upscale_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("lib_renderer upscale bind group layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                            visibility: ShaderStages::VERTEX,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: false },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                            count: None,
                            visibility: ShaderStages::FRAGMENT,
                        },
                    ],
                });

        let upscale_bind_group = ctx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("lib_renderer upscale bind group"),
            layout: &upscale_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: upscale_uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &render_texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let upscale_pipeline = ctx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("lib_renderer upscale pipeline"),
                cache: None,
                depth_stencil: None,
                layout: Some(
                    &ctx.device
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: Some("lib_renderer upscale pipeline layout"),
                            bind_group_layouts: &[&upscale_bind_group_layout],
                            push_constant_ranges: &[],
                        }),
                ),
                multiview: None,
                primitive: PrimitiveState {
                    front_face: FrontFace::Ccw,
                    conservative: false,
                    cull_mode: None,
                    polygon_mode: PolygonMode::Fill,
                    strip_index_format: None,
                    topology: PrimitiveTopology::TriangleList,
                    unclipped_depth: false,
                },
                vertex: VertexState {
                    module: &upscale_shader,
                    entry_point: None,
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[VERTEX_BUFFER_LAYOUT],
                },
                fragment: Some(FragmentState {
                    module: &upscale_shader,
                    targets: &[Some(ColorTargetState {
                        blend: None,
                        format: ctx.surface_format,
                        write_mask: ColorWrites::all(),
                    })],
                    entry_point: None,
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                multisample: MultisampleState::default(),
            });

        Self {
            vertex_buf,
            index_buf,
            render_texture_view,
            quad_uniform_buf,
            quad_bind_group,
            quad_pipeline,
            upscale_uniform_buf,
            upscale_bind_group,
            upscale_pipeline,
        }
    }

    pub fn start_frame<'a>(
        &'a mut self,
        cam: &Camera,
        output: &'a TextureView,
        ctx: RendererContext<'a>,
    ) -> RendererFrame<'a> {
        let quad_uniform = QuadUniform {
            cam_center: (cam.center * PIXELS_PER_UNIT).floor() / PIXELS_PER_UNIT,
        };
        let quad_uniform_bytes =
            unsafe { transmute::<&QuadUniform, &[u8; size_of::<QuadUniform>()]>(&quad_uniform) };

        ctx.queue
            .write_buffer(&self.quad_uniform_buf, 0, quad_uniform_bytes);

        let output_aspect = output.texture().width() as f32 / output.texture().height() as f32;
        let dst_extents = if output_aspect < ASPECT {
            vec2!(1.0, 1.0 * output_aspect / ASPECT)
        } else {
            vec2!(1.0 * ASPECT / output_aspect, 1.0)
        };

        let upscale_uniform = UpscaleUniform {
            src_extents: vec2!(1.0 * cam.ortho_size / MAX_ORTHO_SIZE),
            src_offset: vec2!(0.0),
            dst_extents,
        };
        let upscale_uniform_bytes = unsafe {
            transmute::<&UpscaleUniform, &[u8; size_of::<UpscaleUniform>()]>(&upscale_uniform)
        };

        ctx.queue
            .write_buffer(&self.upscale_uniform_buf, 0, upscale_uniform_bytes);

        RendererFrame {
            renderer: self,
            output,
            ctx,
            ops: Operations {
                load: LoadOp::Clear(Color {
                    r: cam.clear_color.x as f64,
                    g: cam.clear_color.y as f64,
                    b: cam.clear_color.z as f64,
                    a: cam.clear_color.w as f64,
                }),
                store: StoreOp::Store,
            },
        }
    }
}

impl<'a> RendererFrame<'a> {
    pub fn render(&mut self, quads: QuadBufferSlice<'_>) {
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let mut quad_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("lib_renderer quad render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.renderer.render_texture_view,
                    depth_slice: None,
                    ops: self.ops,
                    resolve_target: None,
                })],
            });

            quad_pass.set_vertex_buffer(0, self.renderer.vertex_buf.slice(..));
            quad_pass.set_vertex_buffer(
                1,
                quads.buf.slice(
                    quads.start * size_of::<Quad>() as u64..quads.end * size_of::<Quad>() as u64,
                ),
            );
            quad_pass.set_index_buffer(self.renderer.index_buf.slice(..), IndexFormat::Uint16);
            quad_pass.set_bind_group(0, &self.renderer.quad_bind_group, &[]);
            quad_pass.set_pipeline(&self.renderer.quad_pipeline);

            quad_pass.draw_indexed(0..6, 0, 0..quads.len() as u32);
        }

        self.ctx.queue.submit([encoder.finish()]);
    }
}

impl<'a> Drop for RendererFrame<'a> {
    fn drop(&mut self) {
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let mut upscale_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("lib_renderer upscale render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
                depth_stencil_attachment: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.output,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: StoreOp::Store,
                    },
                    resolve_target: None,
                })],
            });

            upscale_pass.set_vertex_buffer(0, self.renderer.vertex_buf.slice(..));
            upscale_pass.set_index_buffer(self.renderer.index_buf.slice(..), IndexFormat::Uint16);
            upscale_pass.set_bind_group(0, &self.renderer.upscale_bind_group, &[]);
            upscale_pass.set_pipeline(&self.renderer.upscale_pipeline);

            upscale_pass.draw_indexed(0..6, 0, 0..1);
        }

        self.ctx.queue.submit([encoder.finish()]);
    }
}

impl<'a> From<AppContext<'a>> for RendererContext<'a> {
    fn from(value: AppContext<'a>) -> Self {
        Self {
            device: value.device,
            queue: value.queue,
            surface_format: value.surface_format,
        }
    }
}

const VERTEX_BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
    array_stride: size_of::<Vec2f>() as u64,
    step_mode: VertexStepMode::Vertex,
    attributes: &[VertexAttribute {
        format: VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
    }],
};

const QUAD_BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
    array_stride: size_of::<Quad>() as u64,
    step_mode: VertexStepMode::Instance,
    attributes: &[
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: offset_of!(Quad, center) as u64,
            shader_location: 1,
        },
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: (offset_of!(Quad, sprite) + offset_of!(Sprite, center)) as u64,
            shader_location: 2,
        },
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: (offset_of!(Quad, sprite) + offset_of!(Sprite, extents)) as u64,
            shader_location: 3,
        },
        VertexAttribute {
            format: VertexFormat::Float32,
            offset: offset_of!(Quad, layer) as u64,
            shader_location: 4,
        },
    ],
};
