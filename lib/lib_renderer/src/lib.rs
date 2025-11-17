use std::mem::{offset_of, transmute};

mod quad_buffer;
pub use quad_buffer::*;

use lib_gpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType,
    BufferDescriptor, BufferInitDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device,
    DeviceExt, Extent3d, FragmentState, FrontFace, IndexFormat, MultisampleState,
    PipelineCompilationOptions, PolygonMode, PrimitiveState, PrimitiveTopology, Queue,
    RenderPipeline, RenderPipelineDescriptor, SamplerBindingType, ShaderStages, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode, include_wgsl,
};
use lib_math::{
    f32::{FVec2S, FVec4S},
    vec2s,
};

pub struct Renderer {
    vertex_buf: Buffer,
    index_buf: Buffer,
    cam_buf: Buffer,
    bind_group: BindGroup,
    pipeline: RenderPipeline,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub center: FVec2S,
    pub sprite: Sprite,
    pub layer: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    pub min: FVec2S,
    pub size: FVec2S,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub center: FVec2S,
    pub ortho_size: f32,
    pub clear_color: FVec4S,
}

#[derive(Debug, Clone, Copy)]
pub struct RendererContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub surface_format: TextureFormat,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct CameraUniform {
    center: FVec2S,
    extents: FVec2S,
}

impl Renderer {
    pub fn new(ctx: RendererContext<'_>) -> Self {
        let vertex_buf = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("lib_renderer vertex buffer"),
            contents: unsafe {
                transmute::<&[FVec2S; 4], &[u8; 32]>(&[
                    vec2s!(-1.0, -1.0),
                    vec2s!(1.0, -1.0),
                    vec2s!(1.0, 1.0),
                    vec2s!(-1.0, 1.0),
                ])
            },
            usage: BufferUsages::VERTEX,
        });

        let index_buf = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("lib_renderer index buffer"),
            contents: unsafe { transmute::<&[u16; 6], &[u8; 12]>(&[0, 1, 2, 2, 3, 0]) },
            usage: BufferUsages::INDEX,
        });

        let cam_buf = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("lib_renderer camera buffer"),
            size: size_of::<CameraUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let image = {
            let image = image::open("../../../assets/textures/sprites.png")
                .expect("Failed to open lib_renderer texture");

            image.to_rgb8()
        };

        let texture = ctx.device.create_texture(&TextureDescriptor {
            label: Some("lib_renderer texture"),
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

        let bind_group = ctx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("lib_renderer bind group"),
            layout: &ctx
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("lib_renderer bind group layout"),
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::VERTEX,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: false },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                }),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: cam_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &texture.create_view(&TextureViewDescriptor::default()),
                    ),
                },
            ],
        });

        let shader = ctx
            .device
            .create_shader_module(include_wgsl!("shader.wgsl"));

        let pipeline = ctx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("lib_renderer pipeline"),
                cache: None,
                depth_stencil: None,
                layout: None,
                multiview: None,
                primitive: PrimitiveState {
                    front_face: FrontFace::Ccw,
                    conservative: false,
                    cull_mode: None,
                    polygon_mode: PolygonMode::Fill,
                    strip_index_format: Some(IndexFormat::Uint16),
                    topology: PrimitiveTopology::TriangleList,
                    unclipped_depth: false,
                },
                vertex: VertexState {
                    module: &shader,
                    entry_point: None,
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[VERTEX_BUFFER_LAYOUT, INSTANCE_BUFFER_LAYOUT],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    targets: &[Some(ColorTargetState {
                        blend: Some(BlendState::ALPHA_BLENDING),
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
            cam_buf,
            bind_group,
            pipeline,
        }
    }
}

const VERTEX_BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
    array_stride: size_of::<FVec2S>() as u64,
    step_mode: VertexStepMode::Vertex,
    attributes: &[VertexAttribute {
        format: VertexFormat::Float32x2,
        offset: 0,
        shader_location: 4,
    }],
};

const INSTANCE_BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
    array_stride: size_of::<Quad>() as u64,
    step_mode: VertexStepMode::Instance,
    attributes: &[
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: offset_of!(Quad, center) as u64,
            shader_location: 0,
        },
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: (offset_of!(Quad, sprite) + offset_of!(Sprite, min)) as u64,
            shader_location: 1,
        },
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: (offset_of!(Quad, sprite) + offset_of!(Sprite, size)) as u64,
            shader_location: 2,
        },
        VertexAttribute {
            format: VertexFormat::Float32,
            offset: offset_of!(Quad, layer) as u64,
            shader_location: 3,
        },
    ],
};
