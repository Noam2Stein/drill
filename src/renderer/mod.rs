#![expect(unused_imports)]
#![expect(dead_code)]

mod render_buffer;
mod render_frame;
mod render_layer;
mod renderer;
pub use render_buffer::*;
pub use render_frame::*;
pub use render_layer::*;
pub use renderer::*;

const PIXELS_PER_UNIT: f32 = 16.0;
const ASPECT: f32 = 16.0 / 9.0;
const ORTHO_SIZE: f32 = 10.0;
const DYN_QUAD_CAP: usize = 1024;
