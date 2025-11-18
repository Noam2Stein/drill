use std::{
    mem::transmute,
    ops::{Bound, RangeBounds},
};

use lib_gpu::{Buffer, BufferUsages, wgt::BufferDescriptor};

use crate::{Quad, RendererContext};

#[derive(Debug, Clone)]
pub struct QuadBuffer {
    buf: Buffer,
}

#[derive(Debug, Clone, Copy)]
pub struct QuadBufferSlice<'a> {
    pub(crate) buf: &'a Buffer,
    pub(crate) start: u64,
    pub(crate) end: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct QuadBufferRef<'a> {
    buf: &'a Buffer,
    idx: u64,
}

impl QuadBuffer {
    pub fn new(cap: usize, ctx: RendererContext<'_>) -> Self {
        Self {
            buf: ctx.device.create_buffer(&BufferDescriptor {
                label: Some("lib_renderer quad buffer"),
                size: (cap * size_of::<Quad>()) as u64,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        }
    }

    pub fn new_init(quads: &[Quad], ctx: RendererContext<'_>) -> Self {
        let result = Self::new(quads.len(), ctx);
        result.write(quads, ctx);

        result
    }

    pub fn len(&self) -> usize {
        self.buf.size() as usize / size_of::<Quad>()
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> QuadBufferSlice<'_> {
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

        assert!(end <= self.len() as u64);

        QuadBufferSlice {
            buf: &self.buf,
            start,
            end,
        }
    }

    pub fn index(&self, idx: usize) -> QuadBufferRef<'_> {
        assert!(idx < self.len());

        QuadBufferRef {
            buf: &self.buf,
            idx: idx as u64,
        }
    }

    pub fn write(&self, quads: &[Quad], ctx: RendererContext<'_>) {
        self.slice(..).write(quads, ctx)
    }
}

impl<'a> QuadBufferSlice<'a> {
    pub fn len(&self) -> usize {
        (self.end - self.start) as usize
    }

    pub fn slice(&self, range: impl RangeBounds<usize>) -> QuadBufferSlice<'_> {
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

        assert!(end <= self.len() as u64);

        QuadBufferSlice {
            buf: &self.buf,
            start: self.start + start,
            end: self.start + end,
        }
    }

    pub fn index(&self, idx: usize) -> QuadBufferRef<'_> {
        assert!(idx < self.len());

        QuadBufferRef {
            buf: &self.buf,
            idx: self.start + idx as u64,
        }
    }

    pub fn write(&self, quads: &[Quad], ctx: RendererContext<'_>) {
        assert!(quads.len() <= self.len());

        let quads_bytes = unsafe {
            std::slice::from_raw_parts(quads.as_ptr().cast::<u8>(), quads.len() * size_of::<Quad>())
        };

        ctx.queue
            .write_buffer(self.buf, self.start * size_of::<Quad>() as u64, quads_bytes);
    }
}

impl<'a> QuadBufferRef<'a> {
    pub fn write(&self, quad: &Quad, ctx: RendererContext<'_>) {
        let quad_bytes = unsafe { transmute::<&Quad, &[u8; size_of::<Quad>()]>(quad) };

        ctx.queue
            .write_buffer(self.buf, self.idx * size_of::<Quad>() as u64, quad_bytes);
    }
}
