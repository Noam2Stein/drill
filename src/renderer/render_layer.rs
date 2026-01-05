use bytemuck::bytes_of;
use glam::Vec2;
use wgpu::{
    Color, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp,
};

use crate::renderer::{
    DYN_QUAD_CAP, PIXELS_PER_UNIT, Quad, RenderBufferSlice, RenderContext, RenderFrame,
    RenderUniform, Renderer,
};

pub struct RenderLayer<'a> {
    pub(in crate::renderer) renderer: &'a mut Renderer,
    pub(in crate::renderer) ctx: &'a RenderContext<'a>,
    pub(in crate::renderer) has_rendered: &'a mut bool,
}

impl<'a> RenderFrame<'a> {
    pub fn render_layer(&mut self, f: impl FnOnce(&mut RenderLayer), camera_center: Vec2) {
        let render_uniform = RenderUniform {
            cam_center: (camera_center * PIXELS_PER_UNIT).floor() / PIXELS_PER_UNIT,
        };

        self.ctx.queue.write_buffer(
            &self.renderer.render_uniform_buf,
            0,
            bytes_of::<RenderUniform>(&render_uniform),
        );

        f(&mut RenderLayer {
            renderer: self.renderer,
            ctx: self.ctx,
            has_rendered: &mut self.has_rendered,
        })
    }
}

impl<'a> RenderLayer<'a> {
    pub fn render_quad(&mut self, quad: Quad) {
        self.renderer.dyn_quad_vec.push(quad);

        if self.renderer.dyn_quad_vec.len() == DYN_QUAD_CAP {
            self.renderer
                .dyn_quad_buf
                .write(&self.renderer.dyn_quad_vec, *self.ctx);

            self.render_buffer_shared(self.renderer.dyn_quad_buf.slice(..));
            *self.has_rendered = true;

            self.renderer.dyn_quad_vec.clear();
        }
    }

    pub fn render_buffer(&mut self, quads: RenderBufferSlice<'_>) {
        self.render_buffer_shared(quads);
        *self.has_rendered = true;
    }

    fn render_buffer_shared(&self, quads: RenderBufferSlice<'_>) {
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let load_op;
        if *self.has_rendered {
            load_op = LoadOp::Load;
        } else {
            load_op = LoadOp::Clear(Color::BLACK);
        };

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("renderer render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &self.renderer.render_texture,
                ops: Operations {
                    load: load_op,
                    store: StoreOp::Store,
                },
                depth_slice: None,
                resolve_target: None,
            })],
            timestamp_writes: None,
            occlusion_query_set: None,
            depth_stencil_attachment: None,
            multiview_mask: None,
        });

        pass.set_vertex_buffer(0, self.renderer.vertex_buf.slice(..));
        pass.set_vertex_buffer(
            1,
            quads.buf.slice(
                quads.start * size_of::<Quad>() as u64
                    ..(quads.start + quads.len) * size_of::<Quad>() as u64,
            ),
        );
        pass.set_index_buffer(self.renderer.index_buf.slice(..), IndexFormat::Uint16);
        pass.set_bind_group(0, &self.renderer.render_bind_group, &[]);
        pass.set_pipeline(&self.renderer.render_pipeline);

        pass.draw_indexed(0..6, 0, 0..quads.len() as u32);

        drop(pass);

        self.ctx.queue.submit([encoder.finish()]);
    }
}

impl<'a> Drop for RenderLayer<'a> {
    fn drop(&mut self) {
        if self.renderer.dyn_quad_vec.len() > 0 {
            self.renderer
                .dyn_quad_buf
                .write(&self.renderer.dyn_quad_vec, *self.ctx);

            self.render_buffer_shared(
                self.renderer
                    .dyn_quad_buf
                    .slice(..self.renderer.dyn_quad_vec.len()),
            );
            *self.has_rendered = true;

            self.renderer.dyn_quad_vec.clear();
        }
    }
}
