use std::mem::{offset_of, transmute};

mod quad_buffer;
use image::EncodableLayout;
use lib_app::AppContext;
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
    f32::{FVec2S, FVec4S},
    vec2s,
};

#[derive(Debug)]
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
    pub center: FVec2S,
    pub extents: FVec2S,
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
            let image = image::open(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../assets/textures/atlas.png"
            ))
            .expect("Failed to open lib_renderer texture");

            image.to_rgba8()
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

        let shader = ctx
            .device
            .create_shader_module(include_wgsl!("shader.wgsl"));

        let bind_group_layout = ctx
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("lib_renderer bind group layout"),
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

        let bind_group = ctx.device.create_bind_group(&BindGroupDescriptor {
            label: Some("lib_renderer bind group"),
            layout: &bind_group_layout,
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
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let pipeline = ctx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("lib_renderer pipeline"),
                cache: None,
                depth_stencil: None,
                layout: Some(
                    &ctx.device
                        .create_pipeline_layout(&PipelineLayoutDescriptor {
                            label: Some("lib_renderer pipeline layout"),
                            bind_group_layouts: &[&bind_group_layout],
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

    pub fn render(
        &self,
        quads: QuadBufferSlice<'_>,
        cam: &Camera,
        output: &TextureView,
        ctx: RendererContext<'_>,
    ) {
        let aspect = output.texture().width() as f32 / output.texture().height() as f32;

        let cam_uniform = CameraUniform {
            center: cam.center,
            extents: vec2s!(cam.ortho_size * aspect, cam.ortho_size),
        };

        let cam_bytes =
            unsafe { transmute::<&CameraUniform, &[u8; size_of::<CameraUniform>()]>(&cam_uniform) };

        ctx.queue.write_buffer(&self.cam_buf, 0, cam_bytes);

        let mut encoder = ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("lib_renderer render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output,
                depth_slice: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: cam.clear_color.x as f64,
                        g: cam.clear_color.y as f64,
                        b: cam.clear_color.z as f64,
                        a: cam.clear_color.w as f64,
                    }),
                    store: StoreOp::Store,
                },
                resolve_target: None,
            })],
        });

        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.set_vertex_buffer(
            1,
            quads.buf.slice(
                quads.start * size_of::<Quad>() as u64..quads.end * size_of::<Quad>() as u64,
            ),
        );
        pass.set_index_buffer(self.index_buf.slice(..), IndexFormat::Uint16);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_pipeline(&self.pipeline);

        pass.draw_indexed(0..6, 0, 0..quads.len() as u32);

        drop(pass);

        ctx.queue.submit([encoder.finish()]);
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
            offset: (offset_of!(Quad, sprite) + offset_of!(Sprite, center)) as u64,
            shader_location: 1,
        },
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: (offset_of!(Quad, sprite) + offset_of!(Sprite, extents)) as u64,
            shader_location: 2,
        },
        VertexAttribute {
            format: VertexFormat::Float32,
            offset: offset_of!(Quad, layer) as u64,
            shader_location: 3,
        },
    ],
};
