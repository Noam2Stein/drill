use std::time::Duration;

use lib_app::{AppContext, AppEvent, AppFlow, AppHandler};
use lib_gpu::TextureView;
use lib_input::Mapper;
use lib_math::{f32::Vec2f, vec2, vec4};
use lib_renderer::{Camera, DynQuadBuffer, Quad, Renderer, Sprite};

use crate::input::{Input, InputBindings};

#[derive(Debug)]
pub struct Game {
    renderer: Renderer,
    quads: DynQuadBuffer,
    mapper: Mapper<Input>,
    pos: Vec2f,
}

impl AppHandler for Game {
    const TITLE: &str = "Drill";

    fn new(ctx: AppContext<'_>) -> Self {
        Self {
            renderer: Renderer::new(ctx.into()),
            quads: DynQuadBuffer::new(100, ctx.into()),
            mapper: Mapper::new(&InputBindings::default()),
            pos: Vec2f::ZERO,
        }
    }

    fn update(&mut self, delta_time: Duration, _ctx: AppContext<'_>) -> AppFlow {
        let input = self.mapper.map();

        self.pos += vec2!(input.x.value(), input.y.value()) * 10.0 * delta_time.as_secs_f32();

        AppFlow::Continue
    }

    fn event(&mut self, event: AppEvent<'_>, _ctx: AppContext<'_>) -> AppFlow {
        match event {
            AppEvent::Device { device: _, event } => self.mapper.event(event),
            AppEvent::CloseRequested => return AppFlow::Exit,
            _ => {}
        }

        AppFlow::Continue
    }

    fn draw(&mut self, output: &TextureView, ctx: AppContext<'_>) {
        let mut frame = self.renderer.start_frame(
            &Camera {
                center: vec2!(0.0),
                clear_color: vec4!(1.0, 0.0, 0.0, 0.0),
                ortho_size: 8.0,
            },
            output,
            ctx.into(),
        );

        let mut dyn_buffer_frame = self.quads.start_frame(&mut frame);

        dyn_buffer_frame.push(Quad {
            center: self.pos,
            sprite: Sprite {
                center: vec2!(1.0 / 40.0),
                extents: vec2!(1.0 / 40.0),
            },
            layer: 0.0,
        });
    }
}
