use std::ops::{Bound, RangeBounds};

use bytemuck::bytes_of;
use wgpu::{Buffer, BufferDescriptor, BufferUsages};

use crate::renderer::{Quad, RenderContext};

#[derive(Debug, Clone)]
pub struct RenderBuffer(Buffer);

#[derive(Debug, Clone, Copy)]
pub struct RenderBufferSlice<'a> {
    pub(in crate::renderer) buf: &'a Buffer,
    pub(in crate::renderer) start: u64,
    pub(in crate::renderer) len: u64,
}

pub struct RenderBufferRef<'a> {
    buf: &'a Buffer,
    index: u64,
}

impl RenderBuffer {
    pub fn new_uninit(cap: usize, ctx: RenderContext) -> Self {
        Self(ctx.device.create_buffer(&BufferDescriptor {
            label: Some("RendererBuf"),
            size: (cap * size_of::<Quad>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }))
    }

    pub fn new(quads: &[Quad], ctx: RenderContext) -> Self {
        let result = Self::new_uninit(quads.len(), ctx);
        result.write(quads, ctx);

        result
    }

    pub fn len(&self) -> usize {
        self.0.size() as usize / size_of::<Quad>()
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> RenderBufferSlice<'_> {
        let start = match range.start_bound() {
            Bound::Included(start) => *start as u64,
            Bound::Excluded(start) => *start as u64 + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.start_bound() {
            Bound::Included(end) => *end as u64 + 1,
            Bound::Excluded(end) => *end as u64,
            Bound::Unbounded => self.len() as u64,
        };

        assert!(start <= end);
        assert!(end <= self.len() as u64);

        RenderBufferSlice {
            buf: &self.0,
            start,
            len: end - start,
        }
    }

    pub fn index(&self, index: usize) -> RenderBufferRef<'_> {
        assert!(index < self.len());

        RenderBufferRef {
            buf: &self.0,
            index: index as u64,
        }
    }

    pub fn write(&self, quads: &[Quad], ctx: RenderContext) {
        self.slice(..).write(quads, ctx)
    }
}

impl<'a> RenderBufferSlice<'a> {
    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> RenderBufferSlice<'_> {
        let start = match range.start_bound() {
            Bound::Included(start) => *start as u64,
            Bound::Excluded(start) => *start as u64 + 1,
            Bound::Unbounded => 0,
        };

        let end = match range.start_bound() {
            Bound::Included(end) => *end as u64 + 1,
            Bound::Excluded(end) => *end as u64,
            Bound::Unbounded => self.len() as u64,
        };

        assert!(start <= end);
        assert!(end <= self.len() as u64);

        RenderBufferSlice {
            buf: &self.buf,
            start: self.start + start,
            len: end - start,
        }
    }

    pub fn index(&self, index: usize) -> RenderBufferRef<'_> {
        assert!(index < self.len());

        RenderBufferRef {
            buf: &self.buf,
            index: self.start + index as u64,
        }
    }

    pub fn write(&self, quads: &[Quad], ctx: RenderContext<'_>) {
        assert!(quads.len() <= self.len());

        let quads_bytes = unsafe {
            std::slice::from_raw_parts(quads.as_ptr().cast::<u8>(), quads.len() * size_of::<Quad>())
        };

        ctx.queue
            .write_buffer(self.buf, self.start * size_of::<Quad>() as u64, quads_bytes);
    }
}

impl<'a> RenderBufferRef<'a> {
    pub fn write(&self, quad: &Quad, ctx: RenderContext<'_>) {
        let offset = self.index * size_of::<Quad>() as u64;
        let data = bytes_of::<Quad>(quad);

        ctx.queue.write_buffer(self.buf, offset, data);
    }
}
