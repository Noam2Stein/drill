use bytemuck::bytes_of;
use glam::vec2;
use wgpu::{
    Color, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureView,
};

use crate::renderer::{ASPECT, RenderContext, Renderer, UpscaleUniform};

pub struct RenderFrame<'a> {
    pub(in crate::renderer) renderer: &'a mut Renderer,
    pub(in crate::renderer) output: &'a TextureView,
    pub(in crate::renderer) ctx: &'a RenderContext<'a>,
    pub(in crate::renderer) has_rendered: bool,
}

impl Renderer {
    pub fn render_frame(
        &mut self,
        f: impl FnOnce(&mut RenderFrame),
        output: &TextureView,
        ctx: RenderContext,
    ) {
        f(&mut RenderFrame {
            renderer: self,
            output,
            ctx: &ctx,
            has_rendered: false,
        })
    }
}

impl<'a> Drop for RenderFrame<'a> {
    fn drop(&mut self) {
        let output_aspect =
            self.output.texture().width() as f32 / self.output.texture().height() as f32;

        let dst_extents = if output_aspect < ASPECT {
            vec2(1.0, 1.0 * output_aspect / ASPECT)
        } else {
            vec2(1.0 * ASPECT / output_aspect, 1.0)
        };

        let upscale_uniform = UpscaleUniform { dst_extents };

        self.ctx.queue.write_buffer(
            &self.renderer.upscale_uniform_buf,
            0,
            bytes_of::<UpscaleUniform>(&upscale_uniform),
        );

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
                multiview_mask: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: self.output,
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
