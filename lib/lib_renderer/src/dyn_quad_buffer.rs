use crate::{Quad, QuadBuffer, RendererContext, RendererFrame};

#[derive(Debug, Clone)]
pub struct DynQuadBuffer {
    buf: QuadBuffer,
    vec: Vec<Quad>,
}

#[derive(Debug)]
pub struct DynQuadBufferFrame<'a, 'b, 'c> {
    buf: &'a mut DynQuadBuffer,
    frame: &'b mut RendererFrame<'c>,
}

impl DynQuadBuffer {
    pub fn new(cap: usize, ctx: RendererContext<'_>) -> Self {
        Self {
            buf: QuadBuffer::new(cap, ctx),
            vec: Vec::with_capacity(cap),
        }
    }

    pub fn start_frame<'b, 'c>(
        &mut self,
        frame: &'b mut RendererFrame<'c>,
    ) -> DynQuadBufferFrame<'_, 'b, 'c> {
        self.vec.clear();

        DynQuadBufferFrame { buf: self, frame }
    }
}

impl<'a, 'b, 'c> DynQuadBufferFrame<'a, 'b, 'c> {
    pub fn push(&mut self, quad: Quad) {
        self.buf.vec.push(quad);

        if self.buf.vec.len() >= self.buf.buf.len() {
            self.buf.buf.write(&self.buf.vec, self.frame.ctx);
            self.buf.vec.clear();

            self.frame.render(self.buf.buf.slice(..));
        }
    }
}

impl<'a, 'b, 'c> Drop for DynQuadBufferFrame<'a, 'b, 'c> {
    fn drop(&mut self) {
        if !self.buf.vec.is_empty() {
            self.buf.buf.write(&self.buf.vec, self.frame.ctx);
            self.frame.render(self.buf.buf.slice(..self.buf.vec.len()));
        }
    }
}
