use std::mem::offset_of;

use bytemuck::{NoUninit, bytes_of};
use glam::{Vec2, vec2};
use image::EncodableLayout;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device, Extent3d, FilterMode,
    FragmentState, FrontFace, MipmapFilterMode, MultisampleState, Origin3d,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PrimitiveState,
    PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor, SamplerBindingType,
    SamplerDescriptor, ShaderStages, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexState, VertexStepMode, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::{
    asset_path,
    game::GameContext,
    renderer::{ASPECT, DYN_QUAD_CAP, ORTHO_SIZE, PIXELS_PER_UNIT, RenderBuffer},
};

#[derive(Debug)]
pub struct Renderer {
    pub(in crate::renderer) vertex_buf: Buffer,
    pub(in crate::renderer) index_buf: Buffer,
    pub(in crate::renderer) render_uniform_buf: Buffer,
    pub(in crate::renderer) render_bind_group: BindGroup,
    pub(in crate::renderer) render_pipeline: RenderPipeline,
    pub(in crate::renderer) render_texture: TextureView,
    pub(in crate::renderer) upscale_uniform_buf: Buffer,
    pub(in crate::renderer) upscale_bind_group: BindGroup,
    pub(in crate::renderer) upscale_pipeline: RenderPipeline,
    pub(in crate::renderer) dyn_quad_buf: RenderBuffer,
    pub(in crate::renderer) dyn_quad_vec: Vec<Quad>,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub surface_format: TextureFormat,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, NoUninit)]
pub struct Quad {
    pub center: Vec2,
    pub sprite: Sprite,
    pub layer: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, NoUninit)]
pub struct Sprite {
    pub center: Vec2,
    pub extents: Vec2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, NoUninit)]
pub(in crate::renderer) struct RenderUniform {
    pub cam_center: Vec2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, NoUninit)]
pub(in crate::renderer) struct UpscaleUniform {
    pub dst_extents: Vec2,
}

impl Renderer {
    pub fn new(ctx: RenderContext) -> Self {
        let vertex_buf = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("renderer vertex buffer"),
            contents: bytes_of(&[
                vec2(-1.0, -1.0),
                vec2(1.0, -1.0),
                vec2(1.0, 1.0),
                vec2(-1.0, 1.0),
            ]),
            usage: BufferUsages::VERTEX,
        });

        let index_buf = ctx.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("renderer index buffer"),
            contents: bytes_of::<[u16; _]>(&[0, 1, 2, 2, 3, 0]),
            usage: BufferUsages::INDEX,
        });

        let render_texture = ctx
            .device
            .create_texture(&TextureDescriptor {
                label: Some("renderer render texture"),
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                mip_level_count: 1,
                sample_count: 1,
                size: Extent3d {
                    width: (PIXELS_PER_UNIT * ORTHO_SIZE * 2.0 * ASPECT) as u32,
                    height: (PIXELS_PER_UNIT * ORTHO_SIZE * 2.0) as u32,
                    depth_or_array_layers: 1,
                },
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor::default());

        let render_uniform_buf = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("renderer render uniform buffer"),
            size: size_of::<RenderUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sprite_atlas = {
            let image = image::open(asset_path!("sprite_atlas.png"))
                .expect("Failed to open renderer sprites texture")
                .to_rgba8();

            let texture = ctx.device.create_texture(&TextureDescriptor {
                label: Some("renderer sprites texture"),
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
            label: Some("renderer sampler"),
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
            mipmap_filter: MipmapFilterMode::Nearest,
        });

        let render_shader = ctx
            .device
            .create_shader_module(include_wgsl!("render.wgsl"));

        let render_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("renderer render bind group layout"),
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

        let render_bind_group = ctx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("renderer render bind group"),
            layout: &render_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: render_uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(
                        &sprite_atlas.create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let render_pipeline = ctx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("renderer render pipeline"),
                cache: None,
                depth_stencil: None,
                layout: Some(
                    &ctx.device
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: Some("renderer render pipeline layout"),
                            bind_group_layouts: &[&render_bind_group_layout],
                            immediate_size: 0,
                        }),
                ),
                multiview_mask: None,
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
                    module: &render_shader,
                    entry_point: None,
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[VERTEX_BUFFER_LAYOUT, QUAD_BUFFER_LAYOUT],
                },
                fragment: Some(FragmentState {
                    module: &render_shader,
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
            .create_shader_module(include_wgsl!("upscale.wgsl"));

        let upscale_uniform_buf = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("renderer upscale uniform buffer"),
            mapped_at_creation: false,
            size: size_of::<UpscaleUniform>() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        });

        let upscale_bind_group_layout =
            ctx.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("renderer upscale bind group layout"),
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
            label: Some("renderer upscale bind group"),
            layout: &upscale_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: upscale_uniform_buf.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&render_texture),
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
                label: Some("renderer upscale pipeline"),
                cache: None,
                depth_stencil: None,
                layout: Some(
                    &ctx.device
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: Some("renderer upscale pipeline layout"),
                            bind_group_layouts: &[&upscale_bind_group_layout],
                            immediate_size: 0,
                        }),
                ),
                multiview_mask: None,
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

        let dyn_quad_buf = RenderBuffer::new_uninit(DYN_QUAD_CAP, ctx);
        let dyn_quad_vec = Vec::with_capacity(DYN_QUAD_CAP);

        Self {
            vertex_buf,
            index_buf,
            render_texture,
            render_uniform_buf,
            render_bind_group,
            render_pipeline,
            upscale_uniform_buf,
            upscale_bind_group,
            upscale_pipeline,
            dyn_quad_buf,
            dyn_quad_vec,
        }
    }
}

impl<'a> From<GameContext<'a>> for RenderContext<'a> {
    fn from(value: GameContext<'a>) -> Self {
        Self {
            device: value.device,
            queue: value.queue,
            surface_format: value.surface_format,
        }
    }
}

const VERTEX_BUFFER_LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
    array_stride: size_of::<Vec2>() as u64,
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
